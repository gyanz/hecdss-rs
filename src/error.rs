use std::io;
use std::error;
use std::error::Error;
use std::fmt;
use std::ffi::{CStr,CString};
use std::os::raw::*;
use std::str;
use hecdss_sys::*;

pub type DssResult<T> = std::result::Result<T,DssError>;
#[derive(Clone)]
pub struct DssError {
    group:DssErrorGroup,
    kind:DssErrorKind,
    mesg:String,
    c_obj: Box<hec_zdssLastError>,
}

#[derive(Debug,Clone)]
pub enum DssErrorGroup {
    OK,
    ACCESS,
    FILE,
    MEMORY,
    UNKNOWN,
}

#[derive(Debug,Clone)]
pub enum DssErrorKind {
    STATUS_OK,
    INVALID_FILE_VERSION,
    INCOMPATIBLE_VERSION,
    INCOMPATIBLE_VERSION_6,
    INVALID_FILE_NAME,
    NO_EXCLUSIVE_ACCESS,
    UNABLE_TO_ACCESS_FILE,
    UNABLE_TO_WRITE_FILE,
    UNABLE_TO_CREATE_FILE,
    NO_WRITE_PERMISSION,
    NO_PERMISSION,
    WRITE_ON_READ_ONLY,
    INVALID_DSS_FILE,
    INVALID_ADDRESS,
    INVALID_NUMBER_TO_READ,
    INVALID_NUMBER_TO_WRITE,
    WRITE_ERROR,
    READ_ERROR,
    READ_BEYOND_EOF,
    INVALID_FILE_HEADER,
    TRUNCATED_FILE,
    INVALID_HEADER_PARAMETER,
    DAMAGED_FILE,
    CLOSED_FILE,
    EMPTY_FILE,
    IFLTAB_CORRUPT,
    KEY_CORRUPT,
    KEY_VALUE,
    KEY3_LOCATION,
    CANNOT_LOCK_FILE,
    CANNOT_LOCK_EXCLUSIVE,
    CANNOT_LOCK_MULTI_USER,
    CANNOT_SQUEEZE,
    INVALID_BIN_STATUS,
    CANNOT_ALLOCATE_MEMORY,
    INVALID_PARAMETER,
    INVALID_NUMBER,
    INCOMPATIBLE_CALL,
    NON_EMPTY_FILE,
    BIN_SIZE_CONFLICT,
    DIFFERENT_RECORD_TYPE,
    WRONG_RECORD_TYPE,
    NO_UNDELETE_WITH_RECLAIM,
    ARRAY_SPACE_EXHAUSTED,
    BOTH_NOTE_KINDS_USED,
    NO_DATA_GIVEN,
    NO_DATA_READ,
    NO_TIME_WINDOW,
    INVALID_DATE_TIME,
    INVALID_INTERVAL,
    TIMES_NOT_ASCENDING,
    DIFFERENT_PROFILE_NUMBER,
    RECORD_DOES_NOT_EXIST,
    RECORD_ALREADY_EXISTS,
    INVALID_PATHNAME,
    INVALID_RECORD_HEADER,
    ARRAY_TOO_SMALL,
    NOT_OPENED,
    FILE_DOES_NOT_EXIST,
    FILE_EXISTS,
    NULL_FILENAME,
    NULL_PATHNAME,
    NULL_ARGUMENT,
    NULL_ARRAY,
    UNDEFINED_ERROR,
}

// May remove this
#[derive(Debug)]
pub enum _DssErrorType {
    NONE,
    WARNING,
    ACCESS,
    MEMORY,
}

// May remove this
pub enum _DssErrorSeverity {
    INFO,
    WARN,
    INVALID_ARG,
    WRITE_ACCESS,
    FILE_ACCESS,
    WRITE_ERROR,
    READ_ERROR,
    CORRUPT_FILE,
    MEMORY_ERROR,
}

fn kind_from_code(code:i32) -> DssErrorKind {
    match code {
        0 =>  DssErrorKind::STATUS_OK, 
        1 =>  DssErrorKind::INVALID_FILE_VERSION, 
        2 =>  DssErrorKind::INCOMPATIBLE_VERSION,
        3 =>  DssErrorKind::INCOMPATIBLE_VERSION_6,
        4 =>  DssErrorKind::INVALID_FILE_NAME,
        5 =>  DssErrorKind::NO_EXCLUSIVE_ACCESS,
        5 =>  DssErrorKind::UNABLE_TO_ACCESS_FILE,
        7 =>  DssErrorKind::UNABLE_TO_WRITE_FILE,
        8 =>  DssErrorKind::UNABLE_TO_CREATE_FILE,
        9 =>  DssErrorKind::NO_WRITE_PERMISSION,
        10 => DssErrorKind::NO_PERMISSION,
        11 => DssErrorKind::WRITE_ON_READ_ONLY,
        12 => DssErrorKind::INVALID_DSS_FILE,
        13 => DssErrorKind::INVALID_ADDRESS,
        14 => DssErrorKind::INVALID_NUMBER_TO_READ,
        15 => DssErrorKind::INVALID_NUMBER_TO_WRITE,
        16 => DssErrorKind::WRITE_ERROR,
        17 => DssErrorKind::READ_ERROR,
        18 => DssErrorKind::READ_BEYOND_EOF,
        19 => DssErrorKind::INVALID_FILE_HEADER,
        20 => DssErrorKind::TRUNCATED_FILE,
        21 => DssErrorKind::INVALID_HEADER_PARAMETER,
        22 => DssErrorKind::DAMAGED_FILE,
        23 => DssErrorKind::CLOSED_FILE,
        24 => DssErrorKind::EMPTY_FILE,
        25 => DssErrorKind::IFLTAB_CORRUPT,
        26 => DssErrorKind::KEY_CORRUPT,
        27 => DssErrorKind::KEY_VALUE,
        28 => DssErrorKind::KEY3_LOCATION,
        29 => DssErrorKind::CANNOT_LOCK_FILE,
        30 => DssErrorKind::CANNOT_LOCK_EXCLUSIVE,
        31 => DssErrorKind::CANNOT_LOCK_MULTI_USER,
        32 => DssErrorKind::CANNOT_SQUEEZE,
        33 => DssErrorKind::INVALID_BIN_STATUS,
        34 => DssErrorKind::CANNOT_ALLOCATE_MEMORY,
        35 => DssErrorKind::INVALID_PARAMETER,
        36 => DssErrorKind::INVALID_NUMBER,
        37 => DssErrorKind::INCOMPATIBLE_CALL,
        38 => DssErrorKind::NON_EMPTY_FILE,
        39 => DssErrorKind::BIN_SIZE_CONFLICT,
        40 => DssErrorKind::DIFFERENT_RECORD_TYPE,
        41 => DssErrorKind::WRONG_RECORD_TYPE,
        42 => DssErrorKind::NO_UNDELETE_WITH_RECLAIM,
        43 => DssErrorKind::ARRAY_SPACE_EXHAUSTED,
        44 => DssErrorKind::BOTH_NOTE_KINDS_USED,
        45 => DssErrorKind::NO_DATA_GIVEN,
        46 => DssErrorKind::NO_DATA_READ,
        47 => DssErrorKind::NO_TIME_WINDOW,
        48 => DssErrorKind::INVALID_DATE_TIME,
        49 => DssErrorKind::INVALID_INTERVAL,
        50 => DssErrorKind::TIMES_NOT_ASCENDING,
        51 => DssErrorKind::DIFFERENT_PROFILE_NUMBER,
        52 => DssErrorKind::RECORD_DOES_NOT_EXIST,
        53 => DssErrorKind::RECORD_ALREADY_EXISTS,
        54 => DssErrorKind::INVALID_PATHNAME,
        55 => DssErrorKind::INVALID_RECORD_HEADER,
        56 => DssErrorKind::ARRAY_TOO_SMALL,
        57 => DssErrorKind::NOT_OPENED,
        58 => DssErrorKind::FILE_DOES_NOT_EXIST,
        59 => DssErrorKind::FILE_EXISTS,
        60 => DssErrorKind::NULL_FILENAME,
        61 => DssErrorKind::NULL_PATHNAME,
        62 => DssErrorKind::NULL_ARGUMENT,
        63 => DssErrorKind::NULL_ARRAY,
        64 => DssErrorKind::UNDEFINED_ERROR,
        _ => DssErrorKind::UNDEFINED_ERROR,
    }
}

// Methods Implementations
impl DssError {
    pub fn new() -> Self {
        let errobj_ptr= Box::new(hec_zdssLastError{errorCode: 0,
            severity: 0,
            errorNumber: 0,
            errorType: 0,
            systemError: 0,
            lastAddress: 0,
            functionID: 0,
            calledByFunction: 0,
            errorMessage: [0i8;500],
            systemErrorMessage: [0i8;500],
            lastPathname: [0i8;394],
            filename: [0i8;256]
        });
        let egroup = DssErrorGroup::OK;
        let ekind = DssErrorKind::STATUS_OK;
        let emsg = String::from("No Error.");
        DssError{group: egroup, kind: ekind, mesg: emsg, c_obj: errobj_ptr}  
    }

    pub fn raise(mesg:String) -> Self {
        let egroup = DssErrorGroup::UNKNOWN;
        let ekind = DssErrorKind::UNDEFINED_ERROR;
        let mut err = DssError::new();
        err.group = egroup;
        err.kind = ekind;
        err.mesg = mesg;//mesg.to_owned();
        err
    }

    pub fn update(mut self) -> Self {
        // zerrorclear?
        let errobj_ptr = Box::into_raw(self.c_obj);
        unsafe {
            zerror(errobj_ptr);
        }
        let errobj = unsafe {Box::from_raw(errobj_ptr)};
        //let errobj = self.c_obj;
        let c_etype = errobj.errorType; 
        let c_ecode = errobj.errorCode;
        let c_emsg= unsafe {std::mem::transmute::<[i8;500],[u8;500]>(errobj.errorMessage)};      
        let mut egroup = DssErrorGroup::OK;
        let mut ekind:DssErrorKind = kind_from_code(c_ecode); 
        if c_etype < 2 || c_etype > 4{
            // < 2 = NONE and WARNING
            egroup = DssErrorGroup::OK;
        } 
        if c_etype > 1 && c_etype < 5 {
            let egroup:DssErrorGroup = match c_etype {
                    2 => DssErrorGroup::ACCESS,
                    3 => DssErrorGroup::FILE,
                    4 => DssErrorGroup::MEMORY,
                    _ => DssErrorGroup::UNKNOWN
                    };
        }
        let emsg = match str::from_utf8(c_emsg.as_slice()) {
            Ok(data) => data.to_owned(),
            Err(e) => "Error coverting char to utf8".to_string()
        };
        self.group = egroup;
        self.kind = ekind;
        self.mesg = emsg;
        self.c_obj = errobj;
        self
    }

    pub fn is_ok(&self) -> DssResult<()> {
        match self.group {
            DssErrorGroup::OK => Ok(()),
            _ => Err(self.clone())
        }
    }

    pub fn check() -> Result<(),Self> {
        let errobj_ptr= Box::into_raw(Box::new(hec_zdssLastError{errorCode: 0,
            severity: 0,
            errorNumber: 0,
            errorType: 0,
            systemError: 0,
            lastAddress: 0,
            functionID: 0,
            calledByFunction: 0,
            errorMessage: [0i8;500],
            systemErrorMessage: [0i8;500],
            lastPathname: [0i8;394],
            filename: [0i8;256]
        }));
        unsafe {zerror(errobj_ptr);}
        let errobj = unsafe {Box::from_raw(errobj_ptr)};
        let c_etype:i32 = errobj.errorType; 
        let c_ecode:i32 = errobj.errorCode;
        let c_emsg = unsafe {std::mem::transmute::<[i8;500],[u8;500]>(errobj.errorMessage)};
        let egroup = DssErrorGroup::UNKNOWN;
        // Find DssErrorGroup 
        if c_etype < 2 {
            return Ok(())

        } else if c_etype > 1 && c_etype < 5 {
            let egroup:DssErrorGroup = match c_etype {
                    2 => DssErrorGroup::ACCESS,
                    3 => DssErrorGroup::FILE,
                    4 => DssErrorGroup::MEMORY,
                    _ => DssErrorGroup::UNKNOWN,
                    };
        
        } else {
            //egroup = DssErrorGroup::UNKNOWN;
        }

        // Find DssErrorKind
        let ekind:DssErrorKind = kind_from_code(c_ecode); 
        // Copy error message
        let msg = match str::from_utf8(c_emsg.as_slice()) {
            Ok(data) => data.to_owned(),
            Err(e) => "Error coverting char to utf8".to_string()
        };
        // Return
        Err(DssError{group: egroup, kind: ekind, mesg: msg, c_obj: errobj})
    }

    // Other methods below
    //  ...

}


// Traits Implementation
impl fmt::Display for DssError {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f,"Dss error: {}",&self.mesg)
    }
}


impl fmt::Debug for DssError {
    fn fmt(&self, f:&mut fmt::Formatter) -> fmt::Result {
        write!(f,"Dss error: group = {:?}, kind = {:?}, message =  {}",&self.group, &self.kind, &self.mesg)
    }
}


impl error::Error for DssError {}
// fn source(&self) -> Option<&(Error + 'static)> { ... } is optional

impl From<io::Error> for DssError {
    fn from(error:io::Error) -> Self {
        let mut err = DssError::new();
        err.group = DssErrorGroup::UNKNOWN;
        err.kind = DssErrorKind::UNDEFINED_ERROR;
        err.mesg = error.to_string();
        err
    }
}


