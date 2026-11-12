//! failure and （ its is ）。
//!
//! ── Why need need ────────────────────────────────────────────
//! default 127.0.0.1:3065， when bind failure is use up 。
//! 「failure」 has not Reason，：
//! · ** 1 **（`WSAEADDRINUSE` / 10048）—— that can ；
//! · in ****（`WSAEACCES` / 10013）—— no has in 
//! （`netstat` not to ），none from ， can 。
//! if to 1 「 use 」， use will 1 and not in ；
//! 1 「」， will 1 valid 。
//!
//! ── Why is error， not is netsh ────────────────────────
//! `netsh int ipv4 show excludedportrange protocol=tcp` can ，
//! ** not can **： 1 up ， in has can bind success
//! （7901-8000、50000-50059 this can ）， has its in 1 will 。
//! is 「 in netsh 」 not not need ， has bind failure
//! error —— `std::io::ErrorKind` already not 
//! （10048 → `AddrInUse`，10013 → `PermissionDenied`）。
//!
//! not spawn has 1 ： backend.rs ——
//! 「」「create netstat/netsh」 is 。
//!
//! ── ， not ────────────────────────────────────
//! 、 this in backend.rs（ that has Win32 use ），
//! 「 + 」， and use 1 
//! （ not can ： 1 in 、、 not 1 when ，
//! use none from is ）。

use serde::Serialize;

/// 。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConflictKind {
    /// has 1 in ： that can 。
    Occupied,
    /// / ： no has in ， can 。
    Reserved,
    /// its bind failureReason（unavailable、 not etc ）： no has 。
    Other,
}

/// use 。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Occupant {
    pub pid: u32,
    /// （ can filepath file）
    pub name: String,
    /// can file for path； not to when as empty
    pub path: String,
    /// is not is （「 already in 」， not is 「 」）
    pub is_self: bool,
    /// can file is directory（`%SystemRoot%`）。
    ///
    /// this （svchost etc ） not this by ： can can ，
    /// （ can can is ）。 to 「」。
    pub system: bool,
}

/// 1 。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortConflict {
    pub kind: ConflictKind,
    pub port: u16,
    /// to （）
    pub message: String,
    pub occupant: Option<Occupant>,
    /// OS error（ not to ，used for occupant after ）
    #[serde(skip)]
    detail: String,
}

impl PortConflict {
    /// from bind failure error。
    ///
    /// `occupant` by use （backend has Win32 ），
    /// None 「 no 」 or 「 not to 」——`Reserved` not to 。
    pub fn from_bind_error(port: u16, error: &std::io::Error, occupant: Option<Occupant>) -> Self {
        let kind = match error.kind() {
            std::io::ErrorKind::AddrInUse => ConflictKind::Occupied,
            std::io::ErrorKind::PermissionDenied => ConflictKind::Reserved,
            _ => ConflictKind::Other,
        };
        Self::new(kind, port, occupant, &error.to_string())
    }

    /// （ in in this ， and after ）。
    pub fn new(kind: ConflictKind, port: u16, occupant: Option<Occupant>, detail: &str) -> Self {
        let message = describe(kind, port, occupant.as_ref(), detail);
        Self { kind, port, message, occupant, detail: detail.to_string() }
    }

    /// up and 。
    ///
    /// bind failure in `server::start` （ that OS error），「」
    /// need Win32 （ in backend.rs）。 is as error this 
    /// ， —— this 。
    pub fn with_occupant(mut self, occupant: Option<Occupant>) -> Self {
        self.occupant = occupant;
        self.message = describe(self.kind, self.port, self.occupant.as_ref(), &self.detail);
        self
    }

    /// this can not can 「」。
    ///
    /// not kind：directory 、， not this by 。
    /// by —— to 1 failure（ or not this success） by ，
    /// not to 。
    pub fn can_end_occupant(&self) -> bool {
        match &self.occupant {
            Some(who) => self.kind == ConflictKind::Occupied && !who.is_self && !who.system,
            None => false,
        }
    }

    /// that 1 use 。
    ///
    /// and `message` is as not ： has 1 、 need ，
    /// （ can can up ） can title and 。 by after to ，
    /// is as 1 、 and after for not up 。
    pub fn label(&self) -> &'static str {
        match self.kind {
            ConflictKind::Occupied => "端口被占用",
            ConflictKind::Reserved => "端口被系统保留",
            ConflictKind::Other => "网关启动失败",
        }
    }
}

/// by （`new` and `with_occupant` use ， 1 ）
fn describe(kind: ConflictKind, port: u16, occupant: Option<&Occupant>, detail: &str) -> String {
    match kind {
        ConflictKind::Occupied => match occupant {
            Some(who) if who.is_self => format!(
                "端口 {port} 正被本程序占用（PID {}）。网关已在运行，无需重复启动。",
                who.pid
            ),
            Some(who) => format!(
                "端口 {port} 被 {} 占用（PID {}）。结束该进程即可拿回端口。",
                who.name, who.pid
            ),
            None => {
                format!("端口 {port} 已被占用，但查不到是哪个进程在监听（可能是权限不足）。")
            }
        },
        ConflictKind::Reserved => format!(
            "端口 {port} 被系统保留，本机没有任何进程在监听它，因此结束进程无法解决。\
             这类保留段通常由 Hyper-V / WSL2 / Docker 的虚拟网卡服务（winnat、hns）在\
             开机时划分，重启电脑只是碰运气（可能换一段保留，也可能照样撞上）。\
             请改用其它端口。"
        ),
        ConflictKind::Other => format!("端口 {port} 无法监听：{detail}"),
    }
}

/// failure。
///
/// to （`backend:error` + `backend_status` ），
/// ：， is to 「 use 」/「」 。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartupFailure {
    /// to 
    pub message: String,
    /// that 1 use （「 use 」 this ）； not ， and after 1 
    pub label: String,
    /// ；Reason（configfailure、check when ） when as None
    pub conflict: Option<PortConflict>,
    /// this failure can not can （， before in ）
    pub can_end_occupant: bool,
}

impl StartupFailure {
    /// no can success
    pub fn port(conflict: PortConflict) -> Self {
        Self {
            message: conflict.message.clone(),
            label: conflict.label().to_string(),
            can_end_occupant: conflict.can_end_occupant(),
            conflict: Some(conflict),
        }
    }

    /// its Reason failure（config、check etc ）， no has 「」 
    pub fn other(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            label: "网关启动失败".to_string(),
            conflict: None,
            can_end_occupant: false,
        }
    }
}
