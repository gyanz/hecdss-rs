use hecdss_sys::*;
use std::io::prelude::*;
use std::ffi::{CStr,CString};
//use std::os::raw::c_char;
use std::error::Error;
use std::path::Path;
//use std::result;
use std::os::raw::*;
pub mod error;

use error::{DssResult,DssError};


#[derive(Debug)]
pub struct HecDss {
    ifltab: [i64;500],
    filename: String,
    version: i32,
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
                Some(0) => 1,
                Some(1) => 1,
                Some(2) => 1,
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
