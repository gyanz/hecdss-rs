use hecdss_sys::*;
use std::io::prelude::*;
use std::ffi::{CStr,CString};
use std::error::Error;
use std::{mem,str};
use std::os::raw::*;
pub mod error;
use error::{DssResult,DssError};


#[derive(Debug)]
pub struct HecDss {
    ifltab: [i64;500],
    filename: String,
    version: i32,
}

#[derive(Debug)]
pub struct HecTime {
    value: c_int,
    granularity: HecTimeGranularity,
    basedate: c_int
}

#[derive(Debug)]
pub enum HecTimeGranularity {
    second,
    minute,
    hour,
    day
}

impl HecTimeGranularity {
    pub fn default() -> Self {
        HecTimeGranularity::minute
    }

    pub fn value(&self) -> c_int {
        match self {
            HecTimeGranularity::second => self.second_value(),
            HecTimeGranularity::minute => self.minute_value(),
            HecTimeGranularity::hour => self.hour_value(),
            HecTimeGranularity::day => self.day_value()
        } 
    }

    pub fn second_value(&self) -> c_int {
        1
    }

    pub fn minute_value(&self) -> c_int {
        60
    }

    pub fn hour_value(&self) -> c_int {
        3600
    }

    pub fn day_value(&self) -> c_int {
        86400
    }
}

impl HecTime {
    pub fn new(value:c_int,granularity:Option<HecTimeGranularity>,basedate:Option<c_int>) -> Self {
        let gran = match granularity {
            Some(x) => x,
            _ => HecTimeGranularity::default()
        };
        let bdate = match basedate {
            Some(x) => x,
            _ => (0 as c_int)
        };
        HecTime{value:value,granularity:gran,basedate:bdate}
    }

    pub fn from_string(datetime_string:&str) -> Option<Self> {
        let mut julian = [0 as c_int;1];
        let mut sec = [0 as c_int;1];
        let dt = CString::new(datetime_string.to_owned()).unwrap();
        let dt_str = dt.into_raw();
        let status = unsafe {
            spatialDateTime(dt_str,julian.as_mut_slice().as_mut_ptr(),sec.as_mut_slice().as_mut_ptr())
        };
        match status {
            0 => Some(HecTime{value:sec[0],granularity:HecTimeGranularity::second,basedate:julian[0]}),
            _ => {
                println!("HecTime from datatime string returned eroor status {}",status);
                None
            }
        } 
    }

    pub fn to_string(&self) -> Option<(String,String)> {
        let mut cdate = [0 as c_char;13];
        let mut ctime = [0 as c_char;10];
        let granularity = self.granularity.value();
        let cdate_ptr = cdate.as_mut_ptr();
        let ctime_ptr = ctime.as_mut_ptr();
        let status = unsafe{
            getDateAndTime(self.value, granularity,
                 self.basedate, cdate_ptr,
                  mem::size_of::<[c_char;13]>() as c_int,
                  ctime_ptr, mem::size_of::<[c_char;10]>() as c_int)
        };
        match status {
            0 => unsafe {
                //TODO: Replace unwrap with error handling
                let date = CStr::from_ptr(cdate_ptr).to_str().unwrap().to_owned(); 
                let time = CStr::from_ptr(ctime_ptr).to_str().unwrap().to_owned();
                Some((date,time))
                },
            _ => {
                println!("HecTime::to_string return error status {}",status);
                None}
        }
    }

    pub fn add_time(&mut self,time:c_int,unit:HecTimeGranularity) -> Result<(),Box<dyn Error>>{
        let factor = (self.granularity.value() as f32)/(unit.value() as f32);
        let value: c_int = ((time as f32)/factor) as c_int;
        self.value = self.value + value;
        Ok(())
    }
}


impl HecDss {
    pub fn new(dss_file:String) -> Result<Self,Box<dyn Error>> {//DssResult<Self> {
        let mut ifltab = [0i64;500];
        let path = CString::new(dss_file.clone())?;
        unsafe {
            zopen(ifltab.as_mut_slice().as_mut_ptr(),path.as_ptr());
            let _ = DssError::check()?;
            let version = zgetVersion(ifltab.as_mut_slice().as_mut_ptr());
            Ok(HecDss{ifltab: ifltab,
                      filename: dss_file,
                      version: version})
        }
    }

    pub fn read_ts_regular(&mut self,dss_path:&str,retflag:Option<c_int>,dtype:Option<c_int>,alltime:Option<bool>) -> DssResult<Vec<f32>> {
        unsafe {
            let path = CString::new(dss_path).expect("error");
            let zts = zstructTsNew(path.as_ptr());
            let mut err = DssError::new();
            let rflag = match retflag {
                Some(0) => 0,
                Some(-1) => -1,
                _ => -1
            };
            match alltime {
                Some(true) => (*zts).boolRetrieveAllTimes = 1,
                Some(false) => (*zts).boolRetrieveAllTimes = 0,
                _ => (*zts).boolRetrieveAllTimes = 0,
            };

            let float_ordbl = match dtype {
                Some(flag) => 1,
                _ => 1
            };
            let status = ztsRetrieve(self.ifltab.as_mut_slice().as_mut_ptr(),zts,rflag,float_ordbl,0);
            err = err.update();
            err.is_ok()?;
            let mut data = Vec::<f32>::with_capacity((*zts).numberValues as usize);
            let buf_ptr = (*zts).floatValues;
            if !buf_ptr.is_null() {
                let buf = std::slice::from_raw_parts(buf_ptr, (*zts).numberValues as usize);
                for x in buf {
                    data.push(*x);
                }
            }
            Ok(data)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datetime_to_hectime() {
        let datetime = "01JAN2020:1200";
        let hectime = HecTime::from_string(datetime);
        println!("String to Hectime = {:?}",hectime);
    }

    #[test]
    fn hectime_datetime() {
        let hectime = HecTime::new(43200,Some(HecTimeGranularity::second),Some(43830));
        let datetime = hectime.to_string();
        println!("Hectime to string = {:?}",datetime);
    }

    #[test]
    fn read_regular_timeseries() {
        let file_path = String::from("data/example.dss");
        let dss_path = String::from("/REGULAR/TIMESERIES/FLOW//1Hour/Ex1a");
        let mut fid = HecDss::new(file_path).expect("Failed to open HEC-DSS file!");
        let data = fid.read_ts_regular(&dss_path,None,None,None);
        match data {
            Ok(values) => {
                assert_eq!(values.capacity(),12);
                assert_eq!(values[0],450.0);
                println!("Total data = {}",values.capacity());
                println!("Read data = {:?}",values);
            },
            Err(err) => {
                println!("{:?}",err);
                panic!("DssError encountered while reading regular time-series data")}
        }
    }
}
