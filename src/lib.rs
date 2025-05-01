use std::convert::TryFrom;
use std::ffi::CStr;
use std::ffi::CString;
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::num::NonZero;
use std::ops::Deref;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::time::{Duration, SystemTime};

#[cfg(feature = "yang2")]
pub use yang2 as yang;
#[cfg(feature = "yang3")]
pub use yang3 as yang;

use bitflags::bitflags;
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

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct ConnectionFlags: ffi::sr_conn_flag_t::Type {
        const CACHE_RUNNING = ffi::sr_conn_flag_t::SR_CONN_CACHE_RUNNING;
        const SET_PRIV_PARSED = ffi::sr_conn_flag_t::SR_CONN_CTX_SET_PRIV_PARSED;
    }
}

impl Default for ConnectionFlags {
    fn default() -> Self {
        ConnectionFlags::empty()
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Datastore {
    Startup = ffi::sr_datastore_t::SR_DS_STARTUP as isize,
    Running = ffi::sr_datastore_t::SR_DS_RUNNING as isize,
    Candidate = ffi::sr_datastore_t::SR_DS_CANDIDATE as isize,
    Operational = ffi::sr_datastore_t::SR_DS_OPERATIONAL as isize,
    // Available with sysrepo >= 2.2.60
    FactoryDefault = ffi::sr_datastore_t::SR_DS_FACTORY_DEFAULT as isize,
}

impl TryFrom<u32> for Datastore {
    type Error = &'static str;

    fn try_from(t: u32) -> std::result::Result<Self, Self::Error> {
        match t {
            ffi::sr_datastore_t::SR_DS_STARTUP => Ok(Datastore::Startup),
            ffi::sr_datastore_t::SR_DS_RUNNING => Ok(Datastore::Running),
            ffi::sr_datastore_t::SR_DS_CANDIDATE => Ok(Datastore::Candidate),
            ffi::sr_datastore_t::SR_DS_OPERATIONAL => Ok(Datastore::Operational),
            ffi::sr_datastore_t::SR_DS_FACTORY_DEFAULT => Ok(Datastore::FactoryDefault),
            _ => Err("Invalid Datastore"),
        }
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct GetOptions: ffi::sr_get_flag_t::Type {
        const NO_STATE = ffi::sr_get_oper_flag_t::SR_OPER_NO_STATE;
        const NO_CONFIG = ffi::sr_get_oper_flag_t::SR_OPER_NO_CONFIG;
        const NO_SUBS = ffi::sr_get_oper_flag_t::SR_OPER_NO_SUBS;
        const NO_STORED = ffi::sr_get_oper_flag_t::SR_OPER_NO_STORED;
        const WITH_ORIGIN = ffi::sr_get_oper_flag_t::SR_OPER_WITH_ORIGIN;
        // Available with sysrepo >= 2.2.12
        // Prior to sysrepo 2.2.105 was known as as NO_CACHED
        const NO_POLL_CACHED = ffi::sr_get_oper_flag_t::SR_OPER_NO_POLL_CACHED;
        // Available with sysrepo >= 2.2.105
        const NO_RUN_CACHED = ffi::sr_get_oper_flag_t::SR_OPER_NO_RUN_CACHED;
        const NO_FILTER = ffi::sr_get_flag_t::SR_GET_NO_FILTER;
    }
}

impl Default for GetOptions {
    fn default() -> Self {
        GetOptions::empty()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct EditOptions: ffi::sr_edit_flag_t::Type {
        const NON_RECURSIVE = ffi::sr_edit_flag_t::SR_EDIT_NON_RECURSIVE;
        const STRICT = ffi::sr_edit_flag_t::SR_EDIT_STRICT;
        const ISOLATE = ffi::sr_edit_flag_t::SR_EDIT_ISOLATE;
    }
}

impl Default for EditOptions {
    fn default() -> Self {
        EditOptions::empty()
    }
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
    pub struct SubscriptionOptions: ffi::sr_subscr_flag_t::Type {
        const NO_THREAD = ffi::sr_subscr_flag_t::SR_SUBSCR_NO_THREAD;
        const PASSIVE = ffi::sr_subscr_flag_t::SR_SUBSCR_PASSIVE;
        const DONE_ONLY = ffi::sr_subscr_flag_t::SR_SUBSCR_DONE_ONLY;
        const ENABLED = ffi::sr_subscr_flag_t::SR_SUBSCR_ENABLED;
        const UPDATE = ffi::sr_subscr_flag_t::SR_SUBSCR_UPDATE;
        const OPER_MERGE = ffi::sr_subscr_flag_t::SR_SUBSCR_OPER_MERGE;
        // Available with sysrepo >= 2.0.41
        const THREAD_SUSPEND = ffi::sr_subscr_flag_t::SR_SUBSCR_THREAD_SUSPEND,
        // Available with sysrepo >= 2.2.12
        const OPER_POLL_DIFF = ffi::sr_subscr_flag_t::SR_SUBSCR_OPER_POLL_DIFF,
        // Available with sysrepo >= 2.2.150
        const FILTER_ORIG = ffi::sr_subscr_flag_t::SR_SUBSCR_FILTER_ORIG,
        // Available with sysrepo >= 3.3.10
        const CHANGE_ALL_MODULES = ffi::sr_subscr_flag_t::SR_SUBSCR_CHANGE_ALL_MODULES,
    }
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        SubscriptionOptions::empty()
    }
}

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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum NotificationType {
    Realtime = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REALTIME as isize,
    Replay = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY as isize,
    ReplayComplete = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY_COMPLETE as isize,
    Terminated = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_TERMINATED as isize,
    Modified = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_MODIFIED as isize,
    Suspended = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_SUSPENDED as isize,
    Resumed = ffi::sr_ev_notif_type_t::SR_EV_NOTIF_RESUMED as isize,
    // Available with sysrepo >= 2.2.105
    StopTime = ffi::sr_ev_notif_type_t::SR_EV_STOP_TIME as isize,
}

impl TryFrom<ffi::sr_ev_notif_type_t::Type> for NotificationType {
    type Error = &'static str;

    fn try_from(t: ffi::sr_ev_notif_type_t::Type) -> std::result::Result<Self, Self::Error> {
        match t {
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REALTIME => Ok(NotificationType::Realtime),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY => Ok(NotificationType::Replay),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_REPLAY_COMPLETE => {
                Ok(NotificationType::ReplayComplete)
            }
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_TERMINATED => Ok(NotificationType::Terminated),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_MODIFIED => Ok(NotificationType::Modified),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_SUSPENDED => Ok(NotificationType::Suspended),
            ffi::sr_ev_notif_type_t::SR_EV_NOTIF_RESUMED => Ok(NotificationType::Resumed),
            ffi::sr_ev_notif_type_t::SR_EV_STOP_TIME => Ok(NotificationType::StopTime),
            _ => Err("Invalid NotificationType"),
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

/// Do not use *nix's fork(2) after creating a connection.
pub struct Connection {
    conn: *mut ffi::sr_conn_ctx_t,
}

impl Connection {
    pub fn new(flags: ConnectionFlags) -> Result<Self> {
        let mut conn = ptr::null_mut();
        let rc = unsafe { ffi::sr_connect(flags.bits(), &mut conn) };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            debug_assert!(!conn.is_null());
            Ok(Self { conn })
        }
    }

    /// Produce a `Connection` from a raw pointer received from the sysrepo C
    /// API.
    ///
    /// The pointer must not be NULL. Any acquired contexts from this connection
    /// must be released before calling this.
    pub unsafe fn from_raw(conn: *mut ffi::sr_conn_ctx_t) -> Self {
        debug_assert!(!conn.is_null());
        Self { conn }
    }

    pub fn into_raw(self) -> *mut ffi::sr_conn_ctx_t {
        self.conn
    }

    pub fn start_session(&self, ds: Datastore) -> Result<Session<'_>> {
        let mut sess = ptr::null_mut();
        let rc = unsafe { ffi::sr_session_start(self.conn, ds as u32, &mut sess) };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            debug_assert!(!sess.is_null());
            Ok(unsafe { Session::from_raw(self, sess) })
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

impl Drop for Connection {
    fn drop(&mut self) {
        // The sysrepo documentation states that this should be retried until
        // success.
        loop {
            let rc = unsafe { ffi::sr_disconnect(self.conn) };
            let rc = rc as ffi::sr_error_t::Type;
            if rc == ffi::sr_error_t::SR_ERR_OK {
                break;
            }
        }
    }
}

unsafe impl Send for Connection {}
unsafe impl Sync for Connection {}

/// A wrapper around `Context` to ensure it is released back to sysrepo on drop.
pub struct AcquiredContext<'a> {
    conn: &'a Connection,
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

pub struct Session<'a> {
    conn: &'a Connection,
    sess: *mut ffi::sr_session_ctx_t,
}

impl<'a> Session<'a> {
    pub unsafe fn from_raw(conn: &'a Connection, sess: *mut ffi::sr_session_ctx_t) -> Self {
        Self { conn, sess }
    }

    pub fn into_raw(self) -> *mut ffi::sr_session_ctx_t {
        self.sess
    }

    pub fn datastore(&self) -> Datastore {
        Datastore::try_from(unsafe {
            ffi::sr_session_get_ds(self.sess)
        }).expect("datastore from sr_session_get_ds should match a value from sr_datastore_t")
    }

    pub fn switch_datastore(&mut self, datastore: Datastore) -> Result<()> {
        let rc = unsafe { ffi::sr_session_switch_ds(self.sess, datastore as ffi::sr_datastore_t::Type) };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    pub fn get_context(&self) -> Option<AcquiredContext<'a>> {
        self.conn.get_context()
    }

    /// Get a data tree for a given XPath.
    ///
    /// The timeout is rounded to the nearest millisecond.
    pub fn get_data(
        &self,
        xpath: &str,
        max_depth: Option<NonZero<u32>>,
        timeout: Duration,
        options: GetOptions,
    ) -> Result<ManagedData<'a>> {
        let xpath = str_to_cstring(xpath)?;
        let max_depth = max_depth.map(NonZero::get).unwrap_or(0);
        // TODO: double check this actually fits
        let timeout_ms = timeout.as_millis() as u32;
        let mut data: *mut ffi::sr_data_t = ptr::null_mut();

        let rc = unsafe {
            ffi::sr_get_data(
                self.sess,
                xpath.as_ptr(),
                max_depth,
                timeout_ms,
                options.bits(),
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

        unsafe { Ok(ManagedData::from_raw(self.conn, data)) }
    }

    /// Set string item to given Xpath.
    pub fn set_item_str(
        &self,
        path: &str,
        value: &str,
        origin: Option<&str>,
        options: EditOptions,
    ) -> Result<()> {
        let path = str_to_cstring(path)?;
        let value = str_to_cstring(value)?;
        let origin = match origin {
            Some(orig) => Some(str_to_cstring(orig)?),
            None => None,
        };
        let origin_ptr = origin.as_deref().map_or(ptr::null(), |orig| orig.as_ptr());

        let rc = unsafe {
            ffi::sr_set_item_str(
                self.sess,
                path.as_ptr(),
                value.as_ptr(),
                origin_ptr,
                options.bits(),
            )
        };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    /// Apply changes for the session.
    ///
    /// The timeout is rounded to the nearest millisecond.
    pub fn apply_changes(&mut self, timeout: Duration) -> Result<()> {
        // TODO: double check that the duration is short enough
        let timeout_ms = timeout.as_millis() as u32;

        let rc = unsafe { ffi::sr_apply_changes(self.sess, timeout_ms) };
        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    pub fn new_notification_subscription<F>(
        &self,
        mod_name: &str,
        xpath: Option<&str>,
        start_time: Option<SystemTime>,
        stop_time: Option<SystemTime>,
        callback: F,
        options: SubscriptionOptions,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(&Session, u32, NotificationType, &DataTree, SystemTime) + 'static,
    {
        let mut subscr = Subscription::from_raw(self.conn, ptr::null_mut());
        self.notification_subscribe(
            &mut subscr,
            mod_name,
            xpath,
            start_time,
            stop_time,
            callback,
            options,
        )
        .map(|_| subscr)
    }

    pub fn add_notification_subscription<F>(
        &self,
        subscription: &mut Subscription<'a>,
        mod_name: &str,
        xpath: Option<&str>,
        start_time: Option<SystemTime>,
        stop_time: Option<SystemTime>,
        callback: F,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        F: FnMut(&Session, u32, NotificationType, &DataTree, SystemTime) + 'static,
    {
        self.notification_subscribe(
            subscription,
            mod_name,
            xpath,
            start_time,
            stop_time,
            callback,
            options,
        )
    }

    fn notification_subscribe<F>(
        &self,
        subscription: &mut Subscription<'a>,
        mod_name: &str,
        xpath: Option<&str>,
        start_time: Option<SystemTime>,
        stop_time: Option<SystemTime>,
        callback: F,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        // TODO: probably should pass DataNodeRef instead of DataTree
        F: FnMut(&Session, u32, NotificationType, &DataTree, SystemTime) + 'static,
    {
        let mod_name = str_to_cstring(mod_name)?;
        let xpath = match xpath {
            Some(path) => Some(str_to_cstring(path)?),
            None => None,
        };
        let xpath_ptr = xpath.as_deref().map_or(ptr::null(), |xpath| xpath.as_ptr());
        let into_timespec = |t: SystemTime| {
            let d = t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
            timespec {
                tv_sec: d.as_secs() as _,
                tv_nsec: d.subsec_nanos() as _,
            }
        };
        let start_time = start_time
            .map(into_timespec)
            .as_ref()
            .map(|t| t as *const _)
            .unwrap_or(ptr::null());
        let stop_time = stop_time
            .map(into_timespec)
            .as_ref()
            .map(|t| t as *const _)
            .unwrap_or(ptr::null());

        let data = Box::into_raw(Box::new(callback));
        let rc = unsafe {
            ffi::sr_notif_subscribe_tree(
                self.sess,
                mod_name.as_ptr(),
                xpath_ptr,
                start_time,
                stop_time,
                Some(Session::call_event_notif::<F>),
                data as *mut _,
                options.bits(),
                &mut subscription.subscr,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    unsafe extern "C" fn call_event_notif<F>(
        sess: *mut ffi::sr_session_ctx_t,
        sub_id: u32,
        notif_type: ffi::sr_ev_notif_type_t::Type,
        notif: *const yang::ffi::lyd_node,
        timestamp: *mut timespec,
        private_data: *mut c_void,
    ) where
        F: FnMut(&Session, u32, NotificationType, &DataTree, SystemTime),
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let conn = ffi::sr_session_get_connection(sess);
        let ctx = ffi::sr_acquire_context(conn);
        // ctx will never be NULL as the context is locked for reading before
        // this callback is called.
        let ctx = ManuallyDrop::new(Context::from_raw(&(), ctx as *mut _));
        let conn = ManuallyDrop::new(Connection::from_raw(conn));
        let sess = ManuallyDrop::new(Session::from_raw(&conn, sess));
        let notif = ManuallyDrop::new(DataTree::from_raw(&ctx, notif as *mut _));
        let timestamp = timestamp.as_ref().unwrap();
        // These casts are good enough for std.
        let timestamp = SystemTime::UNIX_EPOCH
            + Duration::new(timestamp.tv_sec as u64, timestamp.tv_nsec as u32);
        let notif_type = NotificationType::try_from(notif_type).expect("Convert error");

        callback(&sess, sub_id, notif_type, &notif, timestamp);

        ffi::sr_release_context(conn.conn);
    }

    pub fn new_rpc_subscription<F>(
        &self,
        xpath: &str,
        callback: F,
        priority: u32,
        options: SubscriptionOptions,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(&Session, u32, &str, &DataTree, Event, u32, &mut DataTree) -> Result<()> + 'static,
    {
        let mut subscr = Subscription::from_raw(self.conn, ptr::null_mut());
        self.rpc_subscribe(&mut subscr, xpath, callback, priority, options)
            .map(|_| subscr)
    }

    pub fn add_rpc_subscription<F>(
        &self,
        subscription: &mut Subscription<'a>,
        xpath: &str,
        callback: F,
        priority: u32,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        F: FnMut(&Session, u32, &str, &DataTree, Event, u32, &mut DataTree) -> Result<()> + 'static,
    {
        self.rpc_subscribe(subscription, xpath, callback, priority, options)
    }

    fn rpc_subscribe<F>(
        &self,
        subscription: &mut Subscription<'a>,
        xpath: &str,
        callback: F,
        priority: u32,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        F: FnMut(&Session, u32, &str, &DataTree, Event, u32, &mut DataTree) -> Result<()> + 'static,
    {
        let data = Box::into_raw(Box::new(callback));
        let xpath = str_to_cstring(&xpath)?;

        let rc = unsafe {
            ffi::sr_rpc_subscribe_tree(
                self.sess,
                xpath.as_ptr(),
                Some(Session::call_rpc::<F>),
                data as *mut _,
                priority,
                options.bits(),
                &mut subscription.subscr,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    unsafe extern "C" fn call_rpc<F>(
        sess: *mut ffi::sr_session_ctx_t,
        sub_id: u32,
        op_path: *const c_char,
        input: *const yang::ffi::lyd_node,
        event: ffi::sr_event_t::Type,
        request_id: u32,
        output: *mut yang::ffi::lyd_node,
        private_data: *mut c_void,
    ) -> c_int
    where
        F: FnMut(&Session, u32, &str, &DataTree, Event, u32, &mut DataTree) -> Result<()>,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let op_path = CStr::from_ptr(op_path).to_str().unwrap();
        let conn = ffi::sr_session_get_connection(sess);
        let ctx = ffi::sr_acquire_context(conn);
        // ctx will never be NULL as the context is locked for reading before
        // this callback is called.
        let ctx = ManuallyDrop::new(Context::from_raw(&(), ctx as *mut _));
        let conn = ManuallyDrop::new(Connection::from_raw(conn));
        let sess = ManuallyDrop::new(Session::from_raw(&conn, sess));
        let input = ManuallyDrop::new(DataTree::from_raw(&ctx, input as *mut _));
        let mut output = ManuallyDrop::new(DataTree::from_raw(&ctx, output as *mut _));
        let event = Event::try_from(event).expect("Convert error");

        let res = callback(
            &sess,
            sub_id,
            op_path,
            &input,
            event,
            request_id,
            &mut output,
        );

        ffi::sr_release_context(conn.conn);

        res.err()
            .map(|e| e.errcode)
            .unwrap_or(ffi::sr_error_t::SR_ERR_OK) as c_int
    }

    pub fn new_operational_get_subscription<F>(
        &self,
        mod_name: &str,
        path: &str,
        callback: F,
        options: SubscriptionOptions,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(&Session, u32, &str, &str, Option<&str>, u32, &mut DataTree) -> Result<()>
            + 'static,
    {
        let mut subscr = Subscription::from_raw(self.conn, ptr::null_mut());
        self.oper_get_subscribe(&mut subscr, mod_name, path, callback, options)
            .map(|_| subscr)
    }

    pub fn add_operational_get_subscription<F>(
        &self,
        subscription: &mut Subscription<'a>,
        mod_name: &str,
        path: &str,
        callback: F,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        F: FnMut(&Session, u32, &str, &str, Option<&str>, u32, &mut DataTree) -> Result<()>
            + 'static,
    {
        self.oper_get_subscribe(subscription, mod_name, path, callback, options)
    }

    fn oper_get_subscribe<F>(
        &self,
        subscription: &mut Subscription<'a>,
        mod_name: &str,
        path: &str,
        callback: F,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        F: FnMut(&Session, u32, &str, &str, Option<&str>, u32, &mut DataTree) -> Result<()>
            + 'static,
    {
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
                options.bits(),
                &mut subscription.subscr,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

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
        F: FnMut(&Session, u32, &str, &str, Option<&str>, u32, &mut DataTree) -> Result<()>,
    {
        if private_data.is_null() || parent.is_null() {
            return ffi::sr_error_t::SR_ERR_INTERNAL as c_int;
        }
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let conn = ffi::sr_session_get_connection(sess);
        let ctx = ffi::sr_acquire_context(conn);
        // ctx will never be NULL as the context is locked for reading before
        // this callback is called.
        let ctx = ManuallyDrop::new(Context::from_raw(&(), ctx as *mut _));
        let conn = ManuallyDrop::new(Connection::from_raw(conn));
        let sess = ManuallyDrop::new(Session::from_raw(&conn, sess));
        let mut tree = DataTree::new(&ctx);

        let mod_name = CStr::from_ptr(mod_name).to_str().unwrap();
        let path = CStr::from_ptr(path).to_str().unwrap();
        let request_xpath = if request_xpath.is_null() {
            None
        } else {
            Some(CStr::from_ptr(request_xpath).to_str().unwrap())
        };

        let res = callback(
            &sess,
            sub_id,
            mod_name,
            path,
            request_xpath,
            request_id,
            &mut tree,
        );

        ffi::sr_release_context(conn.conn);

        *parent = tree.into_raw();

        res.err()
            .map(|e| e.errcode)
            .unwrap_or(ffi::sr_error_t::SR_ERR_OK) as c_int
    }

    pub fn new_module_change_subscription<F>(
        &self,
        mod_name: &str,
        xpath: Option<&str>,
        callback: F,
        priority: u32,
        options: SubscriptionOptions,
    ) -> Result<Subscription<'a>>
    where
        F: FnMut(&Session, u32, &str, Option<&str>, Event, u32) -> Result<()> + 'static,
    {
        let mut subscr = Subscription::from_raw(self.conn, ptr::null_mut());
        self.module_change_subscribe(&mut subscr, mod_name, xpath, callback, priority, options)
            .map(|_| subscr)
    }

    pub fn add_module_change_subscription<F>(
        &self,
        subscription: &mut Subscription<'a>,
        mod_name: &str,
        xpath: Option<&str>,
        callback: F,
        priority: u32,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        F: FnMut(&Session, u32, &str, Option<&str>, Event, u32) -> Result<()> + 'static,
    {
        self.module_change_subscribe(subscription, mod_name, xpath, callback, priority, options)
    }

    fn module_change_subscribe<F>(
        &self,
        subscription: &mut Subscription<'a>,
        mod_name: &str,
        xpath: Option<&str>,
        callback: F,
        priority: u32,
        options: SubscriptionOptions,
    ) -> Result<()>
    where
        F: FnMut(&Session, u32, &str, Option<&str>, Event, u32) -> Result<()> + 'static,
    {
        let data = Box::into_raw(Box::new(callback));
        let mod_name = str_to_cstring(mod_name)?;
        let xpath = xpath.map(|p| str_to_cstring(&p)).transpose()?;

        let rc = unsafe {
            ffi::sr_module_change_subscribe(
                self.sess,
                mod_name.as_ptr(),
                xpath.as_deref().map_or(ptr::null(), |p| p.as_ptr()),
                Some(Session::call_module_change::<F>),
                data as *mut _,
                priority,
                options.bits(),
                &mut subscription.subscr,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
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
        F: FnMut(&Session, u32, &str, Option<&str>, Event, u32) -> Result<()>,
    {
        let callback_ptr = private_data as *mut F;
        let callback = &mut *callback_ptr;

        let mod_name = CStr::from_ptr(mod_name).to_str().unwrap();
        let path = if path.is_null() {
            None
        } else {
            Some(CStr::from_ptr(path).to_str().unwrap())
        };
        let event = Event::try_from(event).expect("Convert error");
        let conn = ffi::sr_session_get_connection(sess);
        let conn = ManuallyDrop::new(Connection::from_raw(conn));
        let sess = ManuallyDrop::new(Session::from_raw(&conn, sess));

        let res = callback(&sess, sub_id, mod_name, path, event, request_id);

        res.err()
            .map(|e| e.errcode)
            .unwrap_or(ffi::sr_error_t::SR_ERR_OK) as c_int
    }

    // TODO: only valid in module_change_subscribe callback
    pub fn get_changes_iter(&self, xpath: &str) -> Result<Changes> {
        let xpath = str_to_cstring(xpath)?;
        let mut it = ptr::null_mut();
        let rc = unsafe { ffi::sr_get_changes_iter(self.sess, xpath.as_ptr(), &mut it) };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(unsafe { Changes::from_raw(self, it) })
        }
    }

    /// Send event notify tree.
    pub fn notif_send(&mut self, notif: &DataTree, timeout: Option<Duration>) -> Result<()> {
        let timeout_ms = timeout.map_or(0, |t| t.as_millis() as u32);
        let node = notif.reference().ok_or(Error {
            errcode: ffi::sr_error_t::SR_ERR_INVAL_ARG,
        })?;
        let rc = unsafe {
            ffi::sr_notif_send_tree(
                self.sess,
                node.as_raw(),
                timeout_ms,
                timeout.is_some() as c_int,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            Ok(())
        }
    }

    /// Send RPC.
    pub fn rpc_send(&mut self, input: DataTree<'_>, timeout: Duration) -> Result<ManagedData<'a>> {
        let input = input.into_raw();
        // TODO: check this fits
        let timeout = timeout.as_millis() as u32;

        let mut output = ptr::null_mut();

        let rc = unsafe { ffi::sr_rpc_send_tree(self.sess, input, timeout, &mut output) };

        let rc = rc as ffi::sr_error_t::Type;
        if rc != ffi::sr_error_t::SR_ERR_OK {
            Err(Error { errcode: rc })
        } else {
            unsafe { Ok(ManagedData::from_raw(self.conn, output)) }
        }
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        // The sysrepo documentation states that this should be retried until
        // success.
        loop {
            let rc = unsafe { ffi::sr_session_stop(self.sess) };
            let rc = rc as ffi::sr_error_t::Type;
            if rc == ffi::sr_error_t::SR_ERR_OK {
                break;
            }
        }
    }
}

unsafe impl Send for Session<'_> {}

pub struct ManagedData<'a> {
    ctx: ManuallyDrop<Context>,
    data: *mut ffi::sr_data_t,
    _ghost: PhantomData<&'a ()>,
}

impl<'a> ManagedData<'a> {
    pub unsafe fn from_raw(conn: &'a Connection, data: *mut ffi::sr_data_t) -> Self {
        debug_assert!(!data.is_null());
        // Aquire the context and then drop it right away.
        // SAFETY: This pointer will be valid as the context read lock continues
        // to be held by the data tree.
        let ctx = unsafe {
            let ctx = ffi::sr_acquire_context(conn.conn) as *mut _;
            ffi::sr_release_context(conn.conn);
            ManuallyDrop::new(Context::from_raw(&(), ctx))
        };
        Self {
            ctx,
            data,
            _ghost: PhantomData,
        }
    }

    pub fn into_raw(self) -> *mut ffi::sr_data_t {
        self.data
    }

    pub fn context(&self) -> &Context {
        &self.ctx
    }

    pub fn tree(&self) -> ManagedDataTree<'_> {
        let tree = unsafe { ManuallyDrop::new(DataTree::from_raw(&self.ctx, (*self.data).tree)) };
        ManagedDataTree { tree }
    }
}

impl Drop for ManagedData<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::sr_release_data(self.data);
        }
    }
}

pub struct ManagedDataTree<'a> {
    tree: ManuallyDrop<DataTree<'a>>,
}

impl<'a> Deref for ManagedDataTree<'a> {
    type Target = DataTree<'a>;

    fn deref(&self) -> &DataTree<'a> {
        &self.tree
    }
}

pub struct Subscription<'a> {
    subscr: *mut ffi::sr_subscription_ctx_t,
    _conn: &'a Connection,
}

impl<'a> Subscription<'a> {
    pub fn from_raw(conn: &'a Connection, subscr: *mut ffi::sr_subscription_ctx_t) -> Self {
        Self {
            _conn: conn,
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

pub struct Changes<'a> {
    sess: &'a Session<'a>,
    ctx: ManuallyDrop<Context>,
    iter: *mut ffi::sr_change_iter_t,
}

impl<'a> Changes<'a> {
    pub unsafe fn from_raw(sess: &'a Session<'a>, iter: *mut ffi::sr_change_iter_t) -> Self {
        // Aquire the context and then drop it right away.
        // SAFETY: This pointer will be valid as the context read lock continues
        // to be held by the iterator.
        let ctx = unsafe {
            let ctx = ffi::sr_acquire_context(sess.conn.conn);
            ffi::sr_release_context(sess.conn.conn);
            ManuallyDrop::new(Context::from_raw(&(), ctx as *mut _))
        };
        Self { sess, ctx, iter }
    }

    pub fn iter<'b>(&'b self) -> ChangesIter<'b> {
        ChangesIter {
            sess: self.sess.sess,
            ctx: &self.ctx,
            iter: self.iter,
        }
    }
}

impl Drop for Changes<'_> {
    fn drop(&mut self) {
        unsafe {
            ffi::sr_free_change_iter(self.iter);
        }
    }
}

impl<'a> IntoIterator for &'a Changes<'_> {
    type Item = Result<(ManagedDataTree<'a>, ChangeOperation<'a>)>;
    type IntoIter = ChangesIter<'a>;

    fn into_iter(self) -> ChangesIter<'a> {
        self.iter()
    }
}

pub struct ChangesIter<'a> {
    sess: *mut ffi::sr_session_ctx_t,
    ctx: &'a Context,
    iter: *mut ffi::sr_change_iter_t,
}

impl<'a> Iterator for ChangesIter<'a> {
    // TODO: maybe should be a wrapper around a DataNodeRef instead
    type Item = Result<(ManagedDataTree<'a>, ChangeOperation<'a>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut oper = 0;
        let mut node = ptr::null();
        let mut prev_value = ptr::null();
        let mut prev_list_keys = ptr::null();
        let mut prev_default_flag = 0;

        let rc = unsafe {
            ffi::sr_get_change_tree_next(
                self.sess,
                self.iter,
                &mut oper,
                &mut node,
                &mut prev_value,
                &mut prev_list_keys,
                &mut prev_default_flag,
            )
        };

        let rc = rc as ffi::sr_error_t::Type;
        match rc {
            ffi::sr_error_t::SR_ERR_OK => {
                let node = unsafe { DataTree::from_raw(&self.ctx, node as *mut _) };
                let node = ManagedDataTree {
                    tree: ManuallyDrop::new(node),
                };
                let oper = match oper {
                    ffi::sr_change_oper_t::SR_OP_CREATED if !prev_value.is_null() => {
                        ChangeOperation::CreatedLeafListUserOrdered {
                            previous_value: unsafe { CStr::from_ptr(prev_value).to_str().unwrap() },
                        }
                    }
                    ffi::sr_change_oper_t::SR_OP_CREATED if !prev_list_keys.is_null() => {
                        ChangeOperation::CreatedListUserOrdered {
                            previous_key: unsafe {
                                CStr::from_ptr(prev_list_keys).to_str().unwrap()
                            },
                        }
                    }
                    ffi::sr_change_oper_t::SR_OP_CREATED => ChangeOperation::Created,
                    ffi::sr_change_oper_t::SR_OP_MODIFIED => ChangeOperation::Modified {
                        previous_value: unsafe { CStr::from_ptr(prev_value).to_str().unwrap() },
                        previous_default: prev_default_flag != 0,
                    },
                    ffi::sr_change_oper_t::SR_OP_DELETED => ChangeOperation::Deleted,
                    ffi::sr_change_oper_t::SR_OP_MOVED if !prev_value.is_null() => {
                        ChangeOperation::MovedLeafListUserOrdered {
                            previous_value: unsafe { CStr::from_ptr(prev_value).to_str().unwrap() },
                        }
                    }
                    ffi::sr_change_oper_t::SR_OP_MOVED if !prev_list_keys.is_null() => {
                        ChangeOperation::MovedListUserOrdered {
                            previous_key: unsafe {
                                CStr::from_ptr(prev_list_keys).to_str().unwrap()
                            },
                        }
                    }
                    _ => unreachable!(),
                };
                Some(Ok((node, oper)))
            }
            ffi::sr_error_t::SR_ERR_NOT_FOUND => None,
            _ => Some(Err(Error { errcode: rc })),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ChangeOperation<'a> {
    Created,
    CreatedLeafListUserOrdered {
        previous_value: &'a str,
    },
    CreatedListUserOrdered {
        previous_key: &'a str,
    },
    Modified {
        previous_value: &'a str,
        previous_default: bool,
    },
    Deleted,
    MovedLeafListUserOrdered {
        previous_value: &'a str,
    },
    MovedListUserOrdered {
        previous_key: &'a str,
    },
}

fn str_to_cstring(s: &str) -> Result<CString> {
    CString::new(s).map_err(|_| Error {
        errcode: ffi::sr_error_t::SR_ERR_INVAL_ARG,
    })
}
