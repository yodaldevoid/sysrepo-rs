use std::collections::HashMap;
use std::convert::TryFrom;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::mem::{self, ManuallyDrop};
use std::ops::Deref;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;
use std::time::Duration;
use std::sync::Arc;

#[cfg(feature = "yang2")]
pub use yang2 as yang;
#[cfg(feature = "yang3")]
pub use yang3 as yang;

use libc::{self, size_t};
pub use sysrepo_sys as ffi;
use yang::context::Context;
use yang::data::DataTree;
use yang::ffi::timespec;
use yang::utils::Binding;

/// A convenience wrapper around `Result` for `sysrepo_rs::Error`.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Eq, PartialEq)]
pub struct Error {
    pub errcode: ffi::sr_error_t::Type,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let msg = unsafe { CStr::from_ptr(ffi::sr_strerror(self.errcode as c_int)) };
        write!(f, "{}", String::from_utf8_lossy(msg.to_bytes()))
    }
}

impl std::error::Error for Error {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum LogLevel {
    None = ffi::sr_log_level_t::SR_LL_NONE as isize,
    Error = ffi::sr_log_level_t::SR_LL_ERR as isize,
    Warn = ffi::sr_log_level_t::SR_LL_WRN as isize,
    Info = ffi::sr_log_level_t::SR_LL_INF as isize,
    Debug = ffi::sr_log_level_t::SR_LL_DBG as isize,
}

/// Conn Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ConnFlag {
    Default = ffi::sr_conn_flag_t::SR_CONN_DEFAULT as isize,
    CacheRunning = ffi::sr_conn_flag_t::SR_CONN_CACHE_RUNNING as isize,
}

/// Datastore.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Datastore {
    Startup = ffi::sr_datastore_t::SR_DS_STARTUP as isize,
    Running = ffi::sr_datastore_t::SR_DS_RUNNING as isize,
    Candidate = ffi::sr_datastore_t::SR_DS_CANDIDATE as isize,
    Operational = ffi::sr_datastore_t::SR_DS_OPERATIONAL as isize,
}

/// Sysrepo Type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Type {
    Unknown = ffi::sr_val_type_t::SR_UNKNOWN_T as isize,
    List = ffi::sr_val_type_t::SR_LIST_T as isize,
    Container = ffi::sr_val_type_t::SR_CONTAINER_T as isize,
    ContainerPresence = ffi::sr_val_type_t::SR_CONTAINER_PRESENCE_T as isize,
    LeafEmpty = ffi::sr_val_type_t::SR_LEAF_EMPTY_T as isize,
    Notification = ffi::sr_val_type_t::SR_NOTIFICATION_T as isize,
    Binary = ffi::sr_val_type_t::SR_BINARY_T as isize,
    Bits = ffi::sr_val_type_t::SR_BITS_T as isize,
    Bool = ffi::sr_val_type_t::SR_BOOL_T as isize,
    Decimal64 = ffi::sr_val_type_t::SR_DECIMAL64_T as isize,
    Enum = ffi::sr_val_type_t::SR_ENUM_T as isize,
    IdentityRef = ffi::sr_val_type_t::SR_IDENTITYREF_T as isize,
    InstanceId = ffi::sr_val_type_t::SR_INSTANCEID_T as isize,
    Int8 = ffi::sr_val_type_t::SR_INT8_T as isize,
    Int16 = ffi::sr_val_type_t::SR_INT16_T as isize,
    Int32 = ffi::sr_val_type_t::SR_INT32_T as isize,
    Int64 = ffi::sr_val_type_t::SR_INT64_T as isize,
    String = ffi::sr_val_type_t::SR_STRING_T as isize,
    UInt8 = ffi::sr_val_type_t::SR_UINT8_T as isize,
    UInt16 = ffi::sr_val_type_t::SR_UINT16_T as isize,
    UInt32 = ffi::sr_val_type_t::SR_UINT32_T as isize,
    UInt64 = ffi::sr_val_type_t::SR_UINT64_T as isize,
    AnyXml = ffi::sr_val_type_t::SR_ANYXML_T as isize,
    AnyData = ffi::sr_val_type_t::SR_ANYDATA_T as isize,
}

/// Get Oper Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum GetOperFlag {
    Default = ffi::sr_get_oper_flag_t::SR_OPER_DEFAULT as isize,
    NoState = ffi::sr_get_oper_flag_t::SR_OPER_NO_STATE as isize,
    NoConfig = ffi::sr_get_oper_flag_t::SR_OPER_NO_CONFIG as isize,
    NoSubs = ffi::sr_get_oper_flag_t::SR_OPER_NO_SUBS as isize,
    NoStored = ffi::sr_get_oper_flag_t::SR_OPER_NO_STORED as isize,
    WithOrigin = ffi::sr_get_oper_flag_t::SR_OPER_WITH_ORIGIN as isize,
}

/// Edit Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum EditFlag {
    Default = ffi::sr_edit_flag_t::SR_EDIT_DEFAULT as isize,
    NonRecursive = ffi::sr_edit_flag_t::SR_EDIT_NON_RECURSIVE as isize,
    Strict = ffi::sr_edit_flag_t::SR_EDIT_STRICT as isize,
    Isolate = ffi::sr_edit_flag_t::SR_EDIT_ISOLATE as isize,
}

/// Move Position.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MovePosition {
    Before = ffi::sr_move_position_t::SR_MOVE_BEFORE as isize,
    After = ffi::sr_move_position_t::SR_MOVE_AFTER as isize,
    First = ffi::sr_move_position_t::SR_MOVE_FIRST as isize,
    Last = ffi::sr_move_position_t::SR_MOVE_LAST as isize,
}

/// Subscribe Flag.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum SubcribeFlag {
    Default = ffi::sr_subscr_flag_t::SR_SUBSCR_DEFAULT as isize,
    NoThread = ffi::sr_subscr_flag_t::SR_SUBSCR_NO_THREAD as isize,
    Passive = ffi::sr_subscr_flag_t::SR_SUBSCR_PASSIVE as isize,
    DoneOnly = ffi::sr_subscr_flag_t::SR_SUBSCR_DONE_ONLY as isize,
    Enabled = ffi::sr_subscr_flag_t::SR_SUBSCR_ENABLED as isize,
    Update = ffi::sr_subscr_flag_t::SR_SUBSCR_UPDATE as isize,
    OperMerge = ffi::sr_subscr_flag_t::SR_SUBSCR_OPER_MERGE as isize,
}

/// Event.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Event {
    Update = ffi::sr_event_t::SR_EV_UPDATE as isize,
    Change = ffi::sr_event_t::SR_EV_CHANGE as isize,
    Done = ffi::sr_event_t::SR_EV_DONE as isize,
    Abort = ffi::sr_event_t::SR_EV_ABORT as isize,
    Enabled = ffi::sr_event_t::SR_EV_ENABLED as isize,
    Rpc = ffi::sr_event_t::SR_EV_RPC as isize,
}

impl TryFrom<u32> for Event {
    type Error = &'static str;

    fn try_from(t: u32) -> std::result::Result<Self, Self::Error> {
        match t {
            ffi::sr_event_t::SR_EV_UPDATE => Ok(Event::Update),
            ffi::sr_event_t::SR_EV_CHANGE => Ok(Event::Change),
            ffi::sr_event_t::SR_EV_DONE => Ok(Event::Done),
            ffi::sr_event_t::SR_EV_ABORT => Ok(Event::Abort),
            ffi::sr_event_t::SR_EV_ENABLED => Ok(Event::Enabled),
            ffi::sr_event_t::SR_EV_RPC => Ok(Event::Rpc),
            _ => Err("Invalid Event"),
        }
    }
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Event::Update => "Update",
            Event::Change => "Change",
            Event::Done => "Done",
            Event::Abort => "Abort",
            Event::Enabled => "Enabled",
            Event::Rpc => "RPC",
        };
        write!(f, "{}", s)
    }
}

/// Change Oper.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum ChangeOper {
    Created = ffi::sr_change_oper_t::SR_OP_CREATED as isize,
    Modified = ffi::sr_change_oper_t::SR_OP_MODIFIED as isize,
    Deleted = ffi::sr_change_oper_t::SR_OP_DELETED as isize,
    Moved = ffi::sr_change_oper_t::SR_OP_MOVED as isize,
}

impl TryFrom<u32> for ChangeOper {
    type Error = &'static str;

    fn try_from(t: u32) -> std::result::Result<Self, Self::Error> {
        match t {
            ffi::sr_change_oper_t::SR_OP_CREATED => Ok(ChangeOper::Created),
            ffi::sr_change_oper_t::SR_OP_MODIFIED => Ok(ChangeOper::Modified),
            ffi::sr_change_oper_t::SR_OP_DELETED => Ok(ChangeOper::Deleted),
            ffi::sr_change_oper_t::SR_OP_MOVED => Ok(ChangeOper::Moved),
            _ => Err("Invalid ChangeOper"),
        }
    }
}

impl fmt::Display for ChangeOper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            ChangeOper::Created => "Created",
            ChangeOper::Modified => "Modified",
            ChangeOper::Deleted => "Deleted",
            ChangeOper::Moved => "Moved",
        };
        write!(f, "{}", s)
    }
}

/// Notification Type.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NotifType {
    Realtime = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REALTIME as isize,
    Replay = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY as isize,
    ReplayComplete = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY_COMPLETE as isize,
    Terminated = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_TERMINATED as isize,
    Modified = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_MODIFIED as isize,
    Suspended = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_SUSPENDED as isize,
    Resumed = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_RESUMED as isize,
}

impl TryFrom<u32> for NotifType {
    type Error = &'static str;

    fn try_from(t: u32) -> std::result::Result<Self, Self::Error> {
        match t {
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REALTIME => Ok(NotifType::Realtime),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY => Ok(NotifType::Replay),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY_COMPLETE => Ok(NotifType::ReplayComplete),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_TERMINATED => Ok(NotifType::Terminated),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_MODIFIED => Ok(NotifType::Modified),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_SUSPENDED => Ok(NotifType::Suspended),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_RESUMED => Ok(NotifType::Resumed),
            _ => Err("Invalid NotifType"),
        }
    }
}

/// Typedefs.
pub type SessionId = *const ffi::sr_session_ctx_t;

/// Single Sysrepo Value.
pub struct Value {
    value: *mut ffi::sr_val_t,
}

impl Value {
    pub fn from(value: *mut ffi::sr_val_t) -> Self {
        Self { value: value }
    }

    pub fn value(&self) -> *mut ffi::sr_val_t {
        self.value
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        unsafe {
            ffi::sr_free_val(self.value);
        }
    }
}

/// Slice of Sysrepo Value.
///  The size of slice cannot change.
pub struct ValueSlice {
    /// Pointer to raw sr_val_t array.
    values: *mut ffi::sr_val_t,

    /// Length of this slice.
    len: size_t,

    /// Owned flag.
    owned: bool,
}

impl ValueSlice {
    pub fn new(capacity: size_t, owned: bool) -> Self {
        Self {
            values: unsafe {
                libc::malloc(mem::size_of::<ffi::sr_val_t>() * capacity as usize) as *mut ffi::sr_val_t
            },
            len: capacity,
            owned: owned,
        }
    }

    pub fn from(values: *mut ffi::sr_val_t, len: size_t, owned: bool) -> Self {
        Self {
            values: values,
            len: len,
            owned: owned,
        }
    }

    pub fn at_mut(&mut self, index: usize) -> &mut ffi::sr_val_t {
        let slice = unsafe { slice::from_raw_parts_mut(self.values, self.len as usize) };

        &mut slice[index]
    }

    pub fn as_slice(&mut self) -> &[ffi::sr_val_t] {
        unsafe { slice::from_raw_parts(self.values, self.len as usize) }
    }

    pub fn as_ptr(&self) -> *mut ffi::sr_val_t {
        self.values
    }

    pub fn len(&self) -> size_t {
        self.len
    }

    pub fn set_owned(&mut self) {
        self.owned = true;
    }

    pub fn set_int64_value(&mut self, index: usize, dflt: bool, xpath: &str, value: i64) -> Result<()> {
        let xpath = str_to_cstring(&xpath)?;
        let xpath_ptr = xpath.as_ptr();

        let val = self.at_mut(index) as *mut ffi::sr_val_t;
        unsafe {
            (*val).xpath = libc::strdup(xpath_ptr);
            (*val).type_ = ffi::sr_val_type_t::SR_INT64_T;
            (*val).dflt = if dflt { 0 } else { 1 }; //TODO: It is really those values?
            (*val).data.int64_val = value;
        }

        Ok(())
    }
}

impl Drop for ValueSlice {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                ffi::sr_free_values(self.values, self.len);
            }
        }
    }
}

/// Set logging level for logging to the standard error stream.
pub fn log_stderr(log_level: LogLevel) {
    unsafe {
        ffi::sr_log_stderr(log_level as ffi::sr_log_level_t::Type);
    }
}

/// Set logging level for logging to syslog.
pub fn log_syslog(app_name: &str, log_level: LogLevel) -> Result<()> {
    let app_name = str_to_cstring(app_name)?;
    unsafe {
        ffi::sr_log_syslog(app_name.as_ptr(), log_level as ffi::sr_log_level_t::Type);
    }

    Ok(())
}

/// Sysrepo connection.
pub struct Conn {
    /// Raw Pointer to Connection.
    conn: *mut ffi::sr_conn_ctx_t,

    /// Sessions.
    sessions: HashMap<SessionId, Session>,
}

impl Conn {
    /// Constructor.
    pub fn new(opts: ffi::sr_conn_options_t) -> Result<Conn> {
        let mut conn = ptr::null_mut();

        let rc = unsafe { ffi::sr_connect(opts, &mut conn) };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(Conn {
                conn: conn,
                sessions: HashMap::new(),
            })
        }
    }

    /// Disconnect.
    pub fn disconnect(&mut self) {
        unsafe {
            ffi::sr_disconnect(self.conn);
        }
        self.conn = ptr::null_mut();
    }

    /// Add session to map.
    pub fn insert_session(&mut self, id: SessionId, sess: Session) {
        self.sessions.insert(id, sess);
    }

    /// Add session to map.
    pub fn remove_session(&mut self, id: &SessionId) {
        self.sessions.remove(id);
    }

    /// Lookup session from map.
    pub fn lookup_session(&mut self, id: &SessionId) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    /// Start session.
    pub fn start_session(&mut self, ds: Datastore) -> Result<&mut Session> {
        let mut sess = ptr::null_mut();
        let rc = unsafe { ffi::sr_session_start(self.conn, ds as u32, &mut sess) };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            let id = sess;
            self.insert_session(id, Session::from(sess, true));
            Ok(self.sessions.get_mut(&(id as SessionId)).unwrap())
        }
    }

    pub fn get_context(&self) -> Option<AcquiredContext<'_>> {
        let ctx = unsafe {
            let ctx = ffi::sr_acquire_context(self.conn) as *mut _;
            Context::from_raw_opt(&(), ctx)
        };
        ctx.map(|ctx| AcquiredContext {
            conn: self,
            ctx: ManuallyDrop::new(ctx),
        })
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        self.sessions.drain();
        self.disconnect();
    }
}

/// A wrapper around `Context` to ensure it is released back to sysrepo on drop.
pub struct AcquiredContext<'a> {
    conn: &'a Conn,
    ctx: ManuallyDrop<Context>,
}

impl Deref for AcquiredContext<'_> {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl Drop for AcquiredContext<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::sr_release_context(self.conn.conn);
        }
    }
}

/// Sysrepo session.
pub struct Session {
    /// Raw Pointer to session.
    sess: *mut ffi::sr_session_ctx_t,

    /// Owned flag.
    owned: bool,
}

impl Session {
    /// Constructor.
    pub fn new() -> Self {
        Self {
            sess: ptr::null_mut(),
            owned: true,
        }
    }

    /// Constructor.
    pub fn from(sess: *mut ffi::sr_session_ctx_t, owned: bool) -> Self {
        Self {
            sess: sess,
            owned: owned,
        }
    }

    /// Create unowned clone.
    pub fn clone(&self) -> Self {
        Self {
            sess: self.sess,
            owned: false,
        }
    }

    /// Get raw session context.
    pub unsafe fn get_ctx(&self) -> *mut ffi::sr_session_ctx_t {
        self.sess
    }

    /// Get tree from given XPath.
    pub fn get_data<'a>(
        &mut self,
        context: &'a Arc<Context>,
        xpath: &str,
        max_depth: Option<u32>,
        timeout: Option<Duration>,
        opts: u32
    ) -> Result<DataTree<'a>> {
        let xpath = str_to_cstring(xpath)?;
        let max_depth = max_depth.unwrap_or(0);
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        // SAFETY: data is used as output by sr_get_data and is not read
        let mut data = ptr::null_mut();

        let rc = unsafe {
            ffi::sr_get_data(
                self.sess,
                xpath.as_ptr(),
                max_depth,
                timeout_ms,
                opts,
                &mut data,
            )
        };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            return Err(Error { errcode: rc });
        }
        if data.is_null() {
            return Err(Error {
                errcode: ffi::sr_error_t::SR_ERR_NOT_FOUND,
            });
        }

        let conn = unsafe { ffi::sr_session_get_connection(self.sess) };

        if unsafe { (*data).conn } != conn {
            // It should never happen that the returned connection does not match the supplied one
            // SAFETY: data was checked as not NULL just above
            unsafe {
                ffi::sr_release_data(data);
            }

            return Err(Error {
                errcode: ffi::sr_error_t::SR_ERR_INTERNAL,
            });
        }

        Ok(unsafe { DataTree::from_raw(context, (*data).tree) })
    }

    /// Get items from given Xpath, anre return result in Value slice.
    pub fn get_items(
        &mut self,
        xpath: &str,
        timeout: Option<Duration>,
        opts: u32,
    ) -> Result<ValueSlice> {
        let xpath = str_to_cstring(xpath)?;
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);
        let mut values_count: size_t = 0;
        let mut values = ptr::null_mut();

        let rc = unsafe {
            ffi::sr_get_items(
                self.sess,
                xpath.as_ptr(),
                timeout_ms,
                opts,
                &mut values,
                &mut values_count as *mut size_t,
            )
        };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(ValueSlice::from(values, values_count, true))
        }
    }

    /// Set string item to given Xpath.
    pub fn set_item_str(
        &mut self,
        path: &str,
        value: &str,
        origin: Option<&str>,
        opts: u32,
    ) -> Result<()> {
        let path = str_to_cstring(path)?;
        let value = str_to_cstring(value)?;
        let origin = match origin {
            Some(orig) => Some(str_to_cstring(orig)?),
            None => None,
        };
        let origin_ptr = origin.map_or(ptr::null(), |orig| orig.as_ptr());

        let rc = unsafe {
            ffi::sr_set_item_str(self.sess, path.as_ptr(), value.as_ptr(), origin_ptr, opts)
        };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    /// Apply changes for the session.
    pub fn apply_changes(&mut self, timeout: Option<Duration>) -> Result<()> {
        let timeout_ms = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        let rc = unsafe { ffi::sr_apply_changes(self.sess, timeout_ms) };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    /// Subscribe event notification.
    pub fn notif_subscribe<'a, F>(
        &'a self,
        mod_name: &str,
        xpath: Option<String>,
        start_time: Option<*mut timespec>,
        stop_time: Option<*mut timespec>,
        callback: F,
        opts: ffi::sr_subscr_options_t,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(Session, u32, NotifType, &str, ValueSlice, *mut timespec) + 'static,
    {
        let mod_name = str_to_cstring(mod_name)?;
        let xpath = match xpath {
            Some(path) => Some(str_to_cstring(&path)?),
            None => None,
        };
        let xpath_ptr = xpath.map_or(ptr::null(), |xpath| xpath.as_ptr());
        let start_time = start_time.unwrap_or(ptr::null_mut());
        let stop_time = stop_time.unwrap_or(ptr::null_mut());

        let mut subscr = ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let rc = unsafe {
            ffi::sr_notif_subscribe(
                self.sess,
                mod_name.as_ptr(),
                xpath_ptr,
                start_time,
                stop_time,
                Some(Session::call_event_notif::<F>),
                data as *mut _,
                opts,
                &mut subscr,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(Subscription::from_raw(self, subscr))
        }
    }

    unsafe extern "C" fn call_event_notif<F>(
        sess: *mut ffi::sr_session_ctx_t,
        sub_id: u32,
        notif_type: ffi::sr_ev_notif_type_t::Type,
        path: *const c_char,
        values: *const ffi::sr_val_t,
        values_cnt: size_t,
        timestamp: *mut timespec,
        private_data: *mut c_void,
    ) where
        F: FnMut(Session, u32, NotifType, &str, ValueSlice, *mut timespec),
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let path = CStr::from_ptr(path).to_str().unwrap();
        let sr_values = ValueSlice::from(values as *mut ffi::sr_val_t, values_cnt, false);
        let sess = Session::from(sess, false);
        let notif_type = NotifType::try_from(notif_type).expect("Convert error");

        callback(sess, sub_id, notif_type, path, sr_values, timestamp);
    }

    /// Subscribe RPC.
    pub fn rpc_subscribe<'a, F>(
        &'a self,
        xpath: Option<String>,
        callback: F,
        priority: u32,
        opts: ffi::sr_subscr_options_t,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(Session, u32, &str, ValueSlice, Event, u32) -> ValueSlice + 'static,
    {
        let mut subscr = ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));

        let rc = unsafe {
            match xpath {
                Some(xpath) => {
                    let xpath = str_to_cstring(&xpath)?;
                    ffi::sr_rpc_subscribe(
                        self.sess,
                        xpath.as_ptr(),
                        Some(Session::call_rpc::<F>),
                        data as *mut _,
                        priority,
                        opts,
                        &mut subscr,
                    )
                }
                None => ffi::sr_rpc_subscribe(
                    self.sess,
                    ptr::null_mut(),
                    Some(Session::call_rpc::<F>),
                    data as *mut _,
                    priority,
                    opts,
                    &mut subscr,
                ),
            }
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(Subscription::from_raw(self, subscr))
        }
    }

    unsafe extern "C" fn call_rpc<F>(
        sess: *mut ffi::sr_session_ctx_t,
        sub_id: u32,
        op_path: *const c_char,
        input: *const ffi::sr_val_t,
        input_cnt: size_t,
        event: ffi::sr_event_t::Type,
        request_id: u32,
        output: *mut *mut ffi::sr_val_t,
        output_cnt: *mut size_t,
        private_data: *mut c_void,
    ) -> c_int
    where
        F: FnMut(Session, u32, &str, ValueSlice, Event, u32) -> ValueSlice,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let op_path = CStr::from_ptr(op_path).to_str().unwrap();
        let inputs = ValueSlice::from(input as *mut ffi::sr_val_t, input_cnt, false);
        let sess = Session::from(sess, false);
        let event = Event::try_from(event).expect("Convert error");

        let sr_output = callback(sess, sub_id, op_path, inputs, event, request_id);
        *output = sr_output.as_ptr();
        *output_cnt = sr_output.len();

        ffi::sr_error_t::SR_ERR_OK as c_int
    }

    /// Subscribe oper get items.
    pub fn oper_get_subscribe<'a, F>(
        &'a self,
        mod_name: &str,
        path: &str,
        callback: F,
        opts: ffi::sr_subscr_options_t,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(&mut DataTree<'_>, u32, &str, &str, Option<&str>, u32) + 'static,
    {
        let mut subscr = ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let mod_name = str_to_cstring(mod_name)?;
        let path = str_to_cstring(path)?;

        let rc = unsafe {
            ffi::sr_oper_get_subscribe(
                self.sess,
                mod_name.as_ptr(),
                path.as_ptr(),
                Some(Session::call_get_items::<F>),
                data as *mut _,
                opts,
                &mut subscr,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(Subscription::from_raw(self, subscr))
        }
    }

    // TODO: allow callback to return an error
    unsafe extern "C" fn call_get_items<F>(
        sess: *mut ffi::sr_session_ctx_t,
        sub_id: u32,
        mod_name: *const c_char,
        path: *const c_char,
        request_xpath: *const c_char,
        request_id: u32,
        parent: *mut *mut yang::ffi::lyd_node,
        private_data: *mut c_void,
    ) -> c_int
    where
        F: FnMut(&mut DataTree<'_>, u32, &str, &str, Option<&str>, u32),
    {
        if private_data.is_null() || parent.is_null() {
            return ffi::sr_error_t::SR_ERR_INTERNAL as c_int;
        }
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let conn = ffi::sr_session_get_connection(sess);
        let ctx = ffi::sr_acquire_context(conn);
        // ctx will only be NULL if the context as already locked for writing.
        if ctx.is_null() {
            return ffi::sr_error_t::SR_ERR_LOCKED as c_int;
        }
        let ctx = Context::from_raw(&(), ctx as *mut _);
        let mut tree = DataTree::new(&ctx);

        let mod_name = CStr::from_ptr(mod_name).to_str().unwrap();
        let path = CStr::from_ptr(path).to_str().unwrap();
        let request_xpath = if request_xpath.is_null() {
            None
        } else {
            Some(CStr::from_ptr(request_xpath).to_str().unwrap())
        };

        callback(&mut tree, sub_id, mod_name, path, request_xpath, request_id);

        *parent = tree.into_raw();

        ffi::sr_release_context(conn);

        ffi::sr_error_t::SR_ERR_OK as c_int
    }

    /// Subscribe module change.
    pub fn module_change_subscribe<'a, F>(
        &'a self,
        mod_name: &str,
        path: Option<&str>,
        callback: F,
        priority: u32,
        opts: ffi::sr_subscr_options_t,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(Session, u32, &str, Option<&str>, Event, u32) -> () + 'static,
    {
        let mut subscr = ptr::null_mut();
        let data = Box::into_raw(Box::new(callback));
        let mod_name = str_to_cstring(mod_name)?;
        let path = match path {
            Some(path) => Some(str_to_cstring(&path)?),
            None => None,
        };
        let path_ptr = path.map_or(ptr::null(), |path| path.as_ptr());

        let rc = unsafe {
            ffi::sr_module_change_subscribe(
                self.sess,
                mod_name.as_ptr(),
                path_ptr,
                Some(Session::call_module_change::<F>),
                data as *mut _,
                priority,
                opts,
                &mut subscr,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(Subscription::from_raw(self, subscr))
        }
    }

    unsafe extern "C" fn call_module_change<F>(
        sess: *mut ffi::sr_session_ctx_t,
        sub_id: u32,
        mod_name: *const c_char,
        path: *const c_char,
        event: ffi::sr_event_t::Type,
        request_id: u32,
        private_data: *mut c_void,
    ) -> c_int
    where
        F: FnMut(Session, u32, &str, Option<&str>, Event, u32) -> (),
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let mod_name = CStr::from_ptr(mod_name).to_str().unwrap();
        let path = if path == ptr::null_mut() {
            None
        } else {
            Some(CStr::from_ptr(path).to_str().unwrap())
        };
        let event = Event::try_from(event).expect("Convert error");
        let sess = Session::from(sess, false);

        callback(sess, sub_id, mod_name, path, event, request_id);

        ffi::sr_error_t::SR_ERR_OK as c_int
    }

    /// Get changes iter.
    pub fn get_changes_iter(&self, path: &str) -> Result<ChangeIter> {
        let mut it = ptr::null_mut();

        let path = str_to_cstring(path)?;
        let rc = unsafe { ffi::sr_get_changes_iter(self.sess, path.as_ptr(), &mut it) };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(ChangeIter::from(it))
        }
    }

    /// Send event notify tree.
    pub fn notif_send_tree(&mut self, notif: &DataTree, timeout_ms: u32, wait: bool) -> Result<()> {
        let node = notif.reference().ok_or(Error {
            errcode: ffi::sr_error_t::SR_ERR_INVAL_ARG,
        })?;
        let rc =
            unsafe { ffi::sr_notif_send_tree(self.sess, node.as_raw(), timeout_ms, wait as c_int) };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    /// Send RPC.
    pub fn rpc_send(
        &mut self,
        path: &str,
        input: Option<Vec<ffi::sr_val_t>>,
        timeout: Option<Duration>,
    ) -> Result<ValueSlice> {
        let path = str_to_cstring(path)?;
        let (input, input_cnt) = match input {
            Some(mut input) => (input.as_mut_ptr(), input.len() as size_t),
            None => (ptr::null_mut(), 0),
        };
        let timeout = timeout.map_or(0, |timeout| timeout.as_millis() as u32);

        let mut output = ptr::null_mut();
        let mut output_count: size_t = 0;

        let rc = unsafe {
            ffi::sr_rpc_send(
                self.sess,
                path.as_ptr(),
                input,
                input_cnt,
                timeout,
                &mut output,
                &mut output_count as *mut size_t,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(ValueSlice::from(output, output_count, true))
        }
    }

    /// Return oper, old_value, new_value with next iter.
    pub fn get_change_next(
        &mut self,
        iter: &mut ChangeIter,
    ) -> Option<(ChangeOper, Value, Value)> {
        let mut oper: ffi::sr_change_oper_t::Type = 0;
        let mut old_value: *mut ffi::sr_val_t = ptr::null_mut();
        let mut new_value: *mut ffi::sr_val_t = ptr::null_mut();

        let rc = unsafe {
            ffi::sr_get_change_next(
                self.sess,
                iter.iter(),
                &mut oper,
                &mut old_value,
                &mut new_value,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc == ffi::sr_error_t::SR_ERR_OK {
            match ChangeOper::try_from(oper) {
                Ok(oper) => Some((oper, Value::from(old_value), Value::from(new_value))),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                ffi::sr_session_stop(self.sess);
            }
        }
    }
}

pub struct Subscription<'a> {
    subscr: *mut ffi::sr_subscription_ctx_t,
    _sess: &'a Session,
}

impl<'a> Subscription<'a> {
    pub fn from_raw(sess: &'a Session, subscr: *mut ffi::sr_subscription_ctx_t) -> Self {
        Self {
            _sess: sess,
            subscr,
        }
    }
}

impl Drop for Subscription<'_> {
    fn drop(&mut self) {
        // The sysrepo documentation states that this should be retried until
        // success.
        loop {
            let rc = unsafe { ffi::sr_unsubscribe(self.subscr) };
            let rc = rc as ffi::sr_error_t::Type;
            if rc == ffi::sr_error_t::SR_ERR_OK {
                break;
            }
        }
    }
}

unsafe impl Send for Subscription<'_> {}
unsafe impl Sync for Subscription<'_> {}

/// Sysrepo Changes Iterator.
pub struct ChangeIter {
    /// Raw pointer to iter.
    iter: *mut ffi::sr_change_iter_t,
}

impl ChangeIter {
    pub fn from(iter: *mut ffi::sr_change_iter_t) -> Self {
        Self { iter: iter }
    }

    pub fn iter(&mut self) -> *mut ffi::sr_change_iter_t {
        self.iter
    }
}

impl Drop for ChangeIter {
    fn drop(&mut self) {
        unsafe {
            ffi::sr_free_change_iter(self.iter);
        }
    }
}

fn str_to_cstring(s: &str) -> Result<CString> {
    CString::new(s).map_err(|_| Error {
        errcode: ffi::sr_error_t::SR_ERR_INVAL_ARG,
    })
}
