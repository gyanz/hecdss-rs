use hecdss_sys::*;
use std::io::prelude::*;
use std::{self,mem,str};
use std::ffi::{CStr,CString};
use std::error::Error;
use std::os::raw::*;
pub mod error;
use error::{DssResult,DssError};

#[cfg(feature = "threadsafe")]
use std::sync::Mutex;
#[cfg(feature = "threadsafe")]
use nonparallel::nonparallel;
#[cfg(feature = "threadsafe")]
use lazy_static::lazy_static;
#[cfg(feature = "threadsafe")]
lazy_static! { static ref MUTX: Mutex<()> = Mutex::new(()); }

#[derive(Debug)]
pub struct HecDss {
    ifltab: [i64;500],
    filename: String,
    version: i32,
}

#[derive(Debug)]
pub struct DssPathname {
    apart:Option<String>,
    bpart:Option<String>,
    cpart:Option<String>,
    dpart:Option<String>,
    epart:Option<String>,
    fpart:Option<String>
}

#[derive(Debug,Copy,Clone)]
pub struct HecTime {
    value: c_int,
    granularity: HecTimeGranularity,
    basedate: c_int
}

#[derive(Debug,Copy,Clone)]
pub enum HecTimeGranularity {
    second,
    minute,
    hour,
    day
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum HecTimeInterval {
    second(c_int),
    minute(c_int),
    hour(c_int),
    day(c_int),
    week,
    month,
    semi_month,
    tri_month,
    year,
}

pub enum HecTimeIntervalIreg {
    day,
    month,
    year,
    decade,
    century
}

#[derive(Debug,Copy,Clone)]
pub enum DataType<'a> {
    per_aver,
    per_cum,
    inst_val,
    inst_cum,
    undefined(&'a str)
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum DataUnit<'a> {
    inch,
    feet,
    mm,
    meter,
    cfs,
    cms,
    undefined(&'a str)
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum TimeSeriesType {
    regular,
    irregular
}

#[derive(Debug)]
pub struct TimeSeriesContainer<'a> {
    // all
    ts_type:TimeSeriesType,
    pathname:Option<DssPathname>,
    values:Vec<f32>,
    // meta data
    data_unit:DataUnit<'a>,
    data_type:DataType<'a>,
    // for irregular
    times:Option<Vec<HecTime>>,
    // for regular series only
    start_time:Option<HecTime>,
    interval:Option<HecTimeInterval>
}

#[derive(Debug)]
pub struct TimeSeriesOptions {
    slice:Option<TimeSeriesSlice>,
    trim_start:Option<bool>,
    trim_end:Option<bool>
}

#[derive(Debug)]
pub struct TimeSeriesSlice {
    start_time:Option<HecTime>,
    end_time:Option<HecTime>,
    trim:bool
}

#[derive(Debug)]
pub struct PairedDataTable<'a> {
    pathname:Option<DssPathname>,
    shape:(c_int,c_int),
    headers:Option<Vec<&'a str>>,
    index:Vec<f32>,
    columns:Vec<Vec<f32>>,
    // meta data
    //data_unit:DataUnit<'a>,
    //data_type:DataType<'a>,
    index_unit:DataUnit<'a>,
    index_type:DataType<'a>,
    column_unit:DataUnit<'a>,
    column_type:DataType<'a>
}

#[derive(Debug)]
pub struct PairedDataOptions {
    slice:PairedDataSlice,
}
#[derive(Debug)]
pub struct PairedDataSlice {
    row_start:c_int,
    row_end:Option<c_int>,
    col_start:c_int,
    col_end:Option<c_int>
}

#[derive(Debug)]
pub struct DssMetaData {
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

    pub fn from_value(value:c_int) -> Self {
        match value {
            1 => HecTimeGranularity::second,
            60 => HecTimeGranularity::minute,
            3600 => HecTimeGranularity::hour,
            86400 => HecTimeGranularity::day,
            _ => panic!("Invalid value {} of time granularity",value)
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

impl HecTimeInterval {
    fn to_string(&self) -> String {
        match *self {
            HecTimeInterval::second(x) => format!("{}Second",x),
            HecTimeInterval::minute(x) => format!("{}Minute",x),
            HecTimeInterval::hour(x) => format!("{}Hour",x),
            HecTimeInterval::day(x) => format!("{}Day",x),
            HecTimeInterval::week => format!("Week"),
            HecTimeInterval::month => format!("Month"),
            HecTimeInterval::semi_month => format!("Semi-Month"),
            HecTimeInterval::tri_month => format!("Tri-Month"),
            HecTimeInterval::year => format!("Year"),
        }
    }

    fn value(&self) -> c_int {
        //let sec = (HecTimeGranularity::second).value();
        match *self {
            HecTimeInterval::second(x) => x,
            HecTimeInterval::minute(x) => 60 * x,
            HecTimeInterval::hour(x) => 3600 * x,
            HecTimeInterval::day(x) => 86400 * x,
            HecTimeInterval::week => 604800,
            HecTimeInterval::month => 2592000,
            HecTimeInterval::semi_month => 1296000,
            HecTimeInterval::tri_month => 864000,
            HecTimeInterval::year => 31536000,
        }
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

    fn update(&mut self,hectime:Self) {
        self.value = hectime.value;
        self.granularity = hectime.granularity;
        self.basedate = hectime.basedate;
    }

    //pub fn new_alt(value:c_int,granularity:Option<HecTimeGranularity>,basedate:Option<&str>) -> Self {}

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
        let status = unsafe {
            getDateAndTime(self.value, granularity,
                 self.basedate, cdate_ptr,
                  mem::size_of::<[c_char;13]>() as c_int,
                  ctime_ptr, mem::size_of::<[c_char;10]>() as c_int)
        };
        match status {
            0 => unsafe {
                //"feet".to_string Replace unwrap with error handling
                let date = CStr::from_ptr(cdate_ptr).to_str().unwrap().to_owned(); 
                let time = CStr::from_ptr(ctime_ptr).to_str().unwrap().to_owned();
                Some((date,time))
                },
            _ => {
                println!("HecTime::to_string return error status {}",status);
                None}
        }
    }

    pub fn add_time(&mut self,time:c_int,unit:Option<HecTimeGranularity>) { //-> Result<(),Box<dyn Error>>{
        let factor = match unit {
                            Some(x) => (self.granularity.value() as f32)/(x.value() as f32),
                            _ => 1.0
                        };
        let value: c_int = ((time as f32)/factor) as c_int;
        self.value = self.value + value;
    }

    pub fn date_to_julian(date_string:&str) -> c_int {
        // Converts Date string to days since Julian base date,which is defined by HEC
        // as Dec 31, 1899 (time 00:00 or beginning of day)
        let date = CString::new(date_string.to_owned()).unwrap().into_raw();
        unsafe {
            dateToJulian(date)
        }
    }

    pub fn julian_to_date(days:c_int,fmt:c_int) -> Option<String> {
        let mut cdate = [0 as c_char;13];
        let cdate_ptr = cdate.as_mut_ptr();
        let status = unsafe {
            julianToDate(days, fmt, cdate_ptr, mem::size_of::<[c_char;13]>() as size_t)
        };
        match status {
            0 => unsafe {
                let date = CStr::from_ptr(cdate_ptr).to_str().unwrap().to_owned();
                Some(date)
            },
            _ => None
        }
    }
}

impl <'a> DataType<'a> {
    pub fn to_string(&self) -> String {
        match *self {
            DataType::per_aver => "PER-AVER".to_string(),
            DataType::per_cum => "PER-CUM".to_string(),
            DataType::inst_val => "INST_VAL".to_string(),
            DataType::inst_cum => "INST_CUM".to_string(),
            DataType::undefined(x) => format!("undefined({})",x),
        }
    }

    pub fn from_string(dtype:&'a str) -> Self {
        let value = dtype.to_lowercase();
        match value.as_str() {
            "per-aver" => DataType::per_aver,
            "per-cum" => DataType::per_cum,
            "inst-val" => DataType::inst_val,
            "inst-cum" => DataType::inst_cum,
            _ => DataType::undefined(dtype)
        }
    }
}

impl <'a> DataUnit<'a> {
    pub fn to_string(&self) -> String {
        match *self {
            DataUnit::inch => "inch".to_string(),
            DataUnit::feet => "feet".to_string(),
            DataUnit::mm => "mm".to_string(),
            DataUnit::meter => "meter".to_string(),
            DataUnit::cfs => "cfs".to_string(),
            DataUnit::cms => "cms".to_string(),
            DataUnit::undefined(x) => format!("undefined({})",x)
        }
    }

    pub fn from_string(unit:&'a str) -> Self {
        let value = unit.to_lowercase();
        match value.as_str() {
            "inch" => DataUnit::inch,
            "feet" => DataUnit::feet,
            "mm" => DataUnit::mm,
            "meter" => DataUnit::meter,
            "cfs" => DataUnit::cfs,
            "cms" => DataUnit::cms,
            _ => DataUnit::undefined(unit)
        }
    }
}

impl TimeSeriesType {
    pub fn from_interval(value:c_int) -> Self {
        if value <= 0 {
            TimeSeriesType::irregular
        } else {
            TimeSeriesType::regular
        }
    }

    pub fn value(&self) -> c_int {
        match self {
            TimeSeriesType::irregular => -1,
            TimeSeriesType::regular => 1,
        }
    }
}    

impl <'a> TimeSeriesContainer<'a> {
    pub fn new(ts_type:TimeSeriesType,num_values:c_int) -> Self {
        let mut pathname:Option<DssPathname>= None;
        let mut values = vec![0f32;num_values as usize]; //Vec::<f32>::with_capacity(num_values as usize);
        let mut times :Option<Vec<HecTime>>= None;
        let mut unit = DataUnit::undefined("");
        let mut dtype = DataType::undefined("");
        let mut start_time:Option<HecTime> = None;
        let mut interval:Option<HecTimeInterval> = None;

        match ts_type {
            TimeSeriesType::irregular => {
                times = Some(vec![HecTime::new(0,None,None);num_values as usize]);

            },
            TimeSeriesType::regular => {
                //times = Some(Vec::<HecTime>::with_capacity(1 as usize));
                interval = Some(HecTimeInterval::hour(1));
            }
        };
        TimeSeriesContainer{ts_type:ts_type,
                            pathname:pathname,
                            values:values,
                            data_unit:unit,
                            data_type:dtype,
                            times:times,
                            start_time:start_time,
                            interval:interval}
    }

    pub fn set_pathname(&mut self, path:DssPathname) {
        self.pathname = Some(path);
    }

    pub fn pathname(&self) -> Option<DssPathname> {
        if let Some(ref path) = self.pathname {
            let apart = ((path.apart).as_ref()).cloned();
            let bpart = ((path.bpart).as_ref()).cloned();
            let cpart = ((path.cpart).as_ref()).cloned();
            let dpart = ((path.dpart).as_ref()).cloned();
            let epart = ((path.epart).as_ref()).cloned();
            let fpart = ((path.fpart).as_ref()).cloned();
            return Some(DssPathname::new(apart,bpart,cpart,dpart,epart,fpart))
        }
        None
    }

    pub fn set_values(&mut self, values:&[f32]) -> DssResult<()> {
        let capacity = self.values.len();
        let length = values.len();
        match length {
            capacity => {
                self.values.clone_from_slice(values);
                Ok(())
            },
            _ => Err(DssError::raise("The length of the value is not equal to TimeSeriesContainer capacity".to_string()))?
        }
    }

    pub fn values(&self) -> &[f32] {
        self.values.as_slice()
    }

    pub fn set_times(&mut self, times:&[HecTime]) -> DssResult<()> {
        if let TimeSeriesType::regular = self.ts_type {
            self.start_time = Some(times[0].clone());
            
        } else {
            let capacity = self.values.len();
            let length = times.len();
            if capacity != length {
                Err(DssError::raise(" ".to_string()))?;
            } else {
                if let Some(ref mut moved_time) = self.times {
                    moved_time.clone_from_slice(times);
                };
            }
        }
        Ok(())
    }

    pub fn times(&self,expand_regular:bool) -> Option<Vec<HecTime>>{
        if let TimeSeriesType::regular = self.ts_type {
            if let Some(ref stime) = self.start_time {
                let mut times = Vec::<HecTime>::new();
                let mut start_time = stime.clone();
                if expand_regular {
                    if let Some(ref interval) = self.interval {
                        let interval_sec = interval.clone().value();
                        let loop_count = self.values.len();
                        times.push(start_time);
                        for _ in 1..loop_count {
                            start_time.add_time(interval_sec,Some(HecTimeGranularity::second));
                            times.push(start_time);
                        }
                        
                    } else {
                        panic!("Inverval regular TimeSeries Container not set");
                    }

                } else {
                    times.push(start_time);
                }
                return Some(times)
            }
            
        } else {
            if let Some(ref time) = self.times {
                return Some((*time).iter().map(|x| x.to_owned()).collect())
            }
        }
        None       
    }

    pub fn set_unit(&mut self, unit:&'a str) {
        let unit = DataUnit::from_string(unit);
        self.data_unit = unit;
    }

    pub fn unit(&self) -> DataUnit {
        self.data_unit
    }

    pub fn set_type(&mut self, typ:&'a str) {
        let dtype = DataType::from_string(typ);
        self.data_type = dtype;
    }

    pub fn dtype(&self) ->DataType {
        self.data_type
    }

    pub fn set_interval(&mut self, interval:HecTimeInterval) {
        match self.ts_type {
            TimeSeriesType::regular => self.interval = Some(interval),
            _ => {}
        }
    }

    pub fn interval(&self) -> Option<HecTimeInterval> {
        self.interval
    }

    fn fix_pathname(&mut self) {
        if self.ts_type == TimeSeriesType::regular {
            if let Some(ref mut path) = self.pathname {
                path.dpart = Some(self.interval.as_ref().unwrap().to_string());
            }
        }
    }
}

impl DssPathname {
    pub fn new(a:Option<String>,b:Option<String>,c:Option<String>,d:Option<String>,e:Option<String>,f:Option<String>) -> Self {
        DssPathname{apart:a,bpart:b,cpart:c,dpart:d,epart:e,fpart:f}
    }

    pub fn to_string(&self) -> String {
        let empty = "".to_string();
        format!("/{}/{}/{}/{}/{}/{}/",self.apart.as_ref().unwrap_or(&empty),
                                      self.bpart.as_ref().unwrap_or(&empty),
                                      self.bpart.as_ref().unwrap_or(&empty),
                                      self.dpart.as_ref().unwrap_or(&empty),
                                      self.epart.as_ref().unwrap_or(&empty),
                                      self.fpart.as_ref().unwrap_or(&empty)
                                    )

    }

    pub fn from_string(path:&str) -> Option<Self> {
        if !path.starts_with("/") {return None}
        if !path.ends_with("/") {return None}
        let parts = path.split("/").map(|x| Some(x.trim().to_string())).collect::<Vec<Option<_>>>();
        //println!("{:?}",&parts);
        match parts.len() {
            8 => Some(DssPathname::new(parts[1].clone(),parts[2].clone(),parts[3].clone(),
                                       parts[4].clone(),parts[5].clone(),parts[6].clone())),
            _ => None
        }
    }
}

impl PairedDataSlice {
    pub fn new() -> Self {
        let row_start = 0;
        let col_start = 0;
        let row_end = None;
        let col_end = None;
        PairedDataSlice{row_start:row_start,row_end:row_end,col_start:col_start,col_end:col_end}
    }

    pub fn set_row_range(&mut self,start:c_int,end:c_int) {
        self.row_start = start;
        self.row_end = Some(end);
    }

    pub fn set_col_range(&mut self,start:c_int,end:c_int) {
        self.col_start = start;
        self.col_end = Some(end);
    }
}

impl PairedDataOptions {
    pub fn new() -> Self {
        PairedDataOptions{slice:PairedDataSlice::new()}
    }
}

impl <'a> PairedDataTable<'a> {
    pub fn new(row_count:c_int,col_count:c_int) -> Self {
        let mut pathname:Option<DssPathname>= None;
        let mut columns = vec![vec![0f32;row_count as usize];col_count as usize];
        //let columns = vec![0f32,(row_count as usize),(col_count as usize)];
        let mut index = vec![0f32;row_count as usize];
        //let mut unit = DataUnit::undefined("");
        //let mut dtype = DataType::undefined("");

        PairedDataTable{pathname:pathname,
                        shape:(row_count,col_count),
                        headers:None,
                        index:index,
                        columns:columns,
                        index_unit:DataUnit::undefined(""),
                        index_type:DataType::undefined(""),
                        column_unit:DataUnit::undefined(""),
                        column_type:DataType::undefined("")
                    }
    }

    pub fn set_pathname(&mut self, path:DssPathname) {
        self.pathname = Some(path);
    }

    pub fn set_index(&mut self, values:&[f32]) -> DssResult<()> {
        let (rows,cols) = self.shape;
        let length = (rows*cols) as usize;
        match values.len() {
            length => {
                self.index.clone_from_slice(values);
                Ok(())
            },
            _ => {Err(DssError::raise("The length of the value is not equal to PairedDataTable index capacity".to_string()))?}
        }
    }

    pub fn set_columns(&mut self, values:&[f32]) -> DssResult<()> {
        let (rows,cols) = self.shape;
        let length = (rows*cols) as usize;
        match values.len() {
            length => {
                let mut count = 0;
                for col in 0..(cols as usize) {
                    for row in 0..(rows as usize) {
                        self.columns[col][row] = values[count];
                        count += 1;
                    }
                }
                Ok(())
            },
            _ => Err(DssError::raise("The length of the value is not equal to PairedDataTable column capacity".to_string()))?
        }
    }

    pub fn set_headers(&mut self,headers:Option<Vec<&'a str>>) -> DssResult<()>{
        match headers {
            Some(ref x) => {
                if (*x).len() != (self.shape.1 as usize) {
                    println!("PD Headers = {:?}",headers);
                    Err(DssError::raise(format!("Invalid number ({}) of column header provided",x.len())))?
                };
            },
            _ => {}
        }
        self.headers = headers;
        Ok(())
    }

    pub fn set_index_unit(&mut self,unit:&'a str){
        let unit = DataUnit::from_string(unit);
        self.index_unit = unit;
    }

    pub fn index_unit(&self) -> DataUnit {
        self.index_unit
    }

    pub fn set_index_type(&mut self,typ:&'a str) {
        let typ = DataType::from_string(typ);
        self.index_type = typ;  
    }

    pub fn index_type(&self) -> DataType {
        self.index_type
    }

    pub fn set_column_unit(&mut self,unit:&'a str) {
        let unit = DataUnit::from_string(unit);
        self.column_unit = unit;

    }

    pub fn column_unit(&self) -> DataUnit {
        self.index_unit
    }

    pub fn set_column_type(&mut self,typ:&'a str) {
        let typ = DataType::from_string(typ);
        self.index_type = typ;
    }

    pub fn column_type(&self) -> DataType {
        self.index_type
    }

}

impl HecDss {
    #[cfg_attr(feature="threadsafe",nonparallel(MUTX))]
    pub fn new(dss_file:String) -> Result<Self,Box<dyn Error>> {//DssResult<Self> {
        let mut ifltab = [0i64;500];
        let path = CString::new(dss_file.clone())?;
        let mut err = DssError::new();
        unsafe {
            zopen(ifltab.as_mut_slice().as_mut_ptr(),path.as_ptr());
            err = err.update();
            err.is_ok()?;
            let version = zgetVersion(ifltab.as_mut_slice().as_mut_ptr());
            Ok(HecDss{ifltab: ifltab,
                      filename: dss_file,
                      version: version})
        }
    }

    #[cfg_attr(feature="threadsafe",nonparallel(MUTX))]
    pub fn read_ts(&mut self,dss_path:&str,retflag:Option<c_int>,dtype:Option<c_int>,alltime:Option<bool>) -> DssResult<TimeSeriesContainer> {
        unsafe {
            let path = CString::new(dss_path).expect("error");
            let zts = zstructTsNew(path.as_ptr());
            let mut err = DssError::new();
            let rflag = match retflag {
                Some(0) => 0,
                Some(-1) => -1,
                _ => -1
            };
            // check zts.is_null() ??
            match alltime {
                Some(true) => (*zts).boolRetrieveAllTimes = 1,
                Some(false) => (*zts).boolRetrieveAllTimes = 0,
                _ => (*zts).boolRetrieveAllTimes = 0,
            };

            let float_ordbl = match dtype {
                Some(flag) => 1,
                _ => 1
            };

            // read from dss
            let status = ztsRetrieve(self.ifltab.as_mut_slice().as_mut_ptr(),zts,rflag,float_ordbl,0);
            err = err.update();
            err.is_ok()?;

            // Create the output container
            let interval = (*zts).timeIntervalSeconds;
            let ts_type = TimeSeriesType::from_interval(interval);
            let data_count = (*zts).numberValues as c_int;
            let mut tsc = TimeSeriesContainer::new(ts_type,data_count);
            
            // set meta data here
            let dtype = {
                let dtype = (*zts).type_;
                let datatype = CStr::from_ptr(dtype);
                datatype.to_str().unwrap()
            };
            let unit = {
                let unit = (*zts).units;
                let dataunit = CStr::from_ptr(unit);
                dataunit.to_str().unwrap()
            };
            tsc.set_type(dtype);
            tsc.set_unit(unit);

            // set data
            let buf_ptr:*const f32 = (*zts).floatValues;
            if !buf_ptr.is_null() {
                tsc.set_values(std::slice::from_raw_parts(buf_ptr, data_count as usize));
            }

            // set time values
            let granularity = HecTimeGranularity::from_value((*zts).timeGranularitySeconds);
            let mut basedate = 0 as c_int;
            let mut times = Vec::<HecTime>::with_capacity(data_count as usize);
            match ts_type {
                TimeSeriesType::irregular => {
                    basedate = (*zts).julianBaseDate;
                    let buf_ptr = (*zts).times;
                    if !buf_ptr.is_null() {
                        let buf = std::slice::from_raw_parts(buf_ptr, data_count as usize);
                        for x in buf {
                            times.push(HecTime{value:*x,granularity:granularity,basedate:basedate});
                        }
                        tsc.set_times(times.as_slice());
                    }
                },
                TimeSeriesType::regular => {
                    basedate = (*zts).startJulianDate;
                    let value = (((*zts).startTimeSeconds as f32)/(granularity.value() as f32)) as c_int;
                    let htime = HecTime{value:value,granularity:granularity,basedate:basedate};
                    times.push(htime);
                    tsc.set_times(times.as_slice());
                }
            }
            zstructFree(zts as *mut c_void);
            Ok(tsc)
        }
    }
    
    #[cfg_attr(feature="threadsafe",nonparallel(MUTX))]
    pub fn read_pd(&mut self,dss_path:&str,options:Option<PairedDataOptions>) -> DssResult<PairedDataTable>{
        let dsspath = DssPathname::from_string(dss_path);
        match dsspath {
            Some(ref x) => {},
            None => Err(DssError::raise(format!("Invalid dsspathname {} provided",dss_path)))?
        }
        let path = CString::new(dss_path).expect("Error with dsspath conversion");
        let zpd = unsafe {
            zstructPdNew(path.as_ptr())
        };
        if zpd.is_null() {
            Err(DssError::raise(format!("Error occured with allocation of underlying paired data object")))?;
        }
        let mut rows = 0;
        let mut cols = 0;
        unsafe {
            match options {
                Some(opt) => {
                    let zrs = zstructRecordSizeNew(path.as_ptr());
                    if zrs.is_null() {
                        Err(DssError::raise(format!("Error occured while determining meta data of pd record: {}",dss_path)))?;
                    }
                    let pd_rows = (*zrs).pdNumberOrdinates;
                    let pd_cols = (*zrs).pdNumberCurves;
                    let row_start = opt.slice.row_start;
                    let col_start = opt.slice.col_start;
                    let row_end = opt.slice.row_end.unwrap_or(pd_rows);
                    let col_end = opt.slice.col_end.unwrap_or(pd_cols);
                    if row_start > pd_rows || row_start <1 {
                        Err(DssError::raise(format!("Paired Data row start index {} not in the range 1 - {}",row_start,pd_rows)))?;
                    }
                    if row_end > pd_rows || row_end <1 {
                        Err(DssError::raise(format!("Paired Data row end index {} not in the range 1 - {}",row_end,pd_rows)))?;
                    }
                    if col_start > pd_cols || col_start < 1  {
                        Err(DssError::raise(format!("Paired Data column start index {} not in the range 1 - {}",col_start,pd_cols)))?;
                    }
                    if col_end > pd_cols || col_end < 1  {
                        Err(DssError::raise(format!("Paired Data column end index {} not in the range 1 - {}",col_end,pd_cols)))?;
                    }
                    (*zpd).startingOrdinate = row_start;
                    (*zpd).endingOrdinate = row_end;
                    (*zpd).startingCurve = col_start;
                    (*zpd).endingCurve = col_start;
                },
                None => {}
            }
            let status = zpdRetrieve(self.ifltab.as_mut_ptr(),zpd,1);
            rows = (*zpd).numberOrdinates;
            cols = (*zpd).numberCurves;
        }
        if rows < 1 || cols < 1 {
            Err(DssError::raise(format!("Paired Data has invalid number of rows {} or columns {}",rows,cols)))?;
        }
        let mut ptable = PairedDataTable::new(rows,cols);
        ptable.set_pathname(dsspath.unwrap());
        unsafe {
            let buf_ptr:*const f32 = (*zpd).floatOrdinates;
            ptable.set_index(std::slice::from_raw_parts(buf_ptr, rows as usize));
            let buf_ptr = (*zpd).floatValues;
            ptable.set_columns(std::slice::from_raw_parts(buf_ptr, (rows as usize)*(cols as usize)));
            let label_len:i32 = (*zpd).labelsLength;
            let clabels = std::slice::from_raw_parts((*zpd).labels, label_len as usize);
            let labels = mem::transmute::<&[i8],&[u8]>(clabels);
            let headers = str::from_utf8(labels).unwrap();
            let mut headers:Vec<&str> = headers.trim_end_matches("\x00").split("\x00").collect();
            if headers.len() != (cols as usize) {
                let mut headers_fixed:Vec<&str> = Vec::new();
                let count = std::cmp::min(headers.len(),cols as usize);
                for i in 0..count {
                    headers_fixed.push(headers[i]);
                }
                ptable.set_headers(Some(headers_fixed))?;
            } else {
                ptable.set_headers(Some(headers))?;
            }
            zstructFree(zpd as *mut c_void);
        }
        Ok(ptable)
    }
    

    #[cfg_attr(feature="threadsafe",nonparallel(MUTX))]
    pub fn read_grid(&mut self) {

    }
}

impl Drop for HecDss {
    #[cfg_attr(feature="threadsafe",nonparallel(MUTX))]
    fn drop(&mut self) {
        println!("Freeing the HecDss resource for linked with file: {}",self.filename);
        unsafe {
            zclose(self.ifltab.as_mut_slice().as_mut_ptr());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use std::sync::{Mutex,Arc};
    use std::{thread,time};

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
        let data = fid.read_ts(&dss_path,None,None,None);
        match data {
            Ok(tsc) => {
                let values = tsc.values();
                let times = tsc.times(true);
                let unit = tsc.unit();
                let dtype = tsc.dtype();

                assert_eq!(values.len(),12);
                assert_eq!(values[0],450.0);
                println!("Total data = {}",values.len());
                println!("Read data values = {:?}",values);
                println!("Data unit = {:?}, type = {:?}",unit,dtype);
                let time_strings:Vec<(String,String)> = times.unwrap().iter()
                                                             .map(|x| match x.to_string() {
                                                                Some(x) => x,
                                                                _ => ("".to_string(),"".to_string())
                                                                })
                                                             .collect();
                println!("Read data times = {:?}",time_strings);
                                                    
            },
            Err(err) => {
                println!("{:?}",err);
                panic!("DssError encountered while reading regular time-series data")}
        }
    }

    #[test]
    fn read_regular_timeseries_mthread() {
        println!("==========================================");
        println!("Running functions from multiple threads");
        println!("==========================================");
        let file_path = "data/example.dss";
        let dss_path = "/REGULAR/TIMESERIES/FLOW//1Hour/Ex1";
        let dss_patha = "/REGULAR/TIMESERIES/FLOW//1Hour/Ex1a";
        let mut handles = vec![];

        let handle1 = thread::spawn(move || {
            println!("**Starting thread 1");
            let mut fid = HecDss::new(file_path.to_string()).expect("Failed to open HEC-DSS file!");
            let data = fid.read_ts(dss_path,None,None,None);
            thread::sleep(time::Duration::from_millis(10));
            println!("**Thread 1 TSC = {:?}",data);
            println!("**End of thread 1");
        });

        let handle2 = thread::spawn(move || {
            println!("^^Starting thread 2");
            let mut fid = HecDss::new(file_path.to_string()).expect("Failed to open HEC-DSS file!");
            let data = fid.read_ts(dss_patha,None,None,None);
            println!("^^Thread 2 TSC = {:?}",data);
            println!("^^End of thread 2");
        });

        handles.push(handle1);
        handles.push(handle2);

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn read_paired_data() {
        let file_path = String::from("data/example.dss");
        let dss_path = String::from("/PAIREDDATA/PTABLE/FREQ-FLOW///Ex2/");
        let mut fid = HecDss::new(file_path).expect("Failed to open HEC-DSS file!");
        let options = None;
        let data = fid.read_pd(&dss_path,options);
        println!("Paired data table = {:?}",data);
    }

}
