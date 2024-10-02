use std::path::Path;

use crate::style::ModeChar;

#[derive(Default, Debug, Clone)]
pub struct Attributes {
    pub archivable: bool,
    pub readonly: bool,
    pub hidden: bool,
    pub system: bool,
    #[cfg(target_os = "windows")]
    pub executable: bool
}

impl From<&Path> for Attributes {
    fn from(value: &Path) -> Self {
        #[cfg(target_os = "windows")]
        return {
            use std::os::windows::ffi::OsStrExt;
            use windows::core::PCWSTR;
            use windows::Win32::Storage::FileSystem::{
                GetBinaryTypeW, GetFileAttributesW, FILE_ATTRIBUTE_ARCHIVE,
                FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_READONLY,
                FILE_ATTRIBUTE_SYSTEM,
            };

            let path = value
                .as_os_str()
                .encode_wide()
                .map(|v| if v == 47 { 92 } else { v })
                .chain([0])
                .collect::<Vec<_>>();

            let attrs = unsafe { GetFileAttributesW(PCWSTR::from_raw(path.as_ptr())) };
            let mut binary_type = 0u32;

            Self {
                executable: unsafe { GetBinaryTypeW(PCWSTR::from_raw(path.as_ptr()), &mut binary_type as *mut _).is_ok() },
                archivable: attrs & FILE_ATTRIBUTE_ARCHIVE.0 == FILE_ATTRIBUTE_ARCHIVE.0,
                readonly: attrs & FILE_ATTRIBUTE_READONLY.0 == FILE_ATTRIBUTE_READONLY.0,
                hidden: attrs & FILE_ATTRIBUTE_HIDDEN.0 == FILE_ATTRIBUTE_HIDDEN.0,
                system: attrs & FILE_ATTRIBUTE_SYSTEM.0 == FILE_ATTRIBUTE_SYSTEM.0,
            }
        };

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        return Self::default()
    }
}

#[derive(Default, Debug, Clone)]
pub struct Perms {
    user: User,
    group: Group,
    everyone: Group,
    attributes: Attributes,
}
impl Perms {
    pub fn is_hidden(&self) -> bool {
        self.attributes.hidden
    }

    pub fn user(&self) -> &User {
        &self.user
    }

    pub fn group(&self) -> &Group {
        &self.group
    }

    pub fn everyone(&self) -> &Group {
        &self.everyone
    }

    pub fn attributes(&self) -> &Attributes {
        &self.attributes
    }
}

impl std::fmt::Display for Perms {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.user.permissions,
            self.group.permissions,
            self.everyone.permissions,
        )
    }
}

impl TryFrom<&Path> for Perms {
    type Error = Box<dyn std::error::Error>;
    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            use std::os::unix::fs::{PermissionsExt, MetadataExt};
            let meta = value.metadata().unwrap();
            let permissions = meta.permissions();
            let st_mode = permissions.mode();

            let user = users::get_user_by_uid(meta.uid());
            let group = users::get_group_by_gid(meta.gid());
            Ok(Self {
                user: User {
                    domain: Default::default(),
                    name: user.map(|usr| usr.name().to_string_lossy().to_string()).unwrap_or_default(),
                    permissions: AccessRights(((st_mode & 0b111 << 6)>>6) as u8),
                },
                group: Group::new("", group.map(|grp| grp.name().to_string_lossy().to_string()).unwrap_or_default(), AccessRights(((st_mode & 0b111 << 3)>>3) as u8)),
                everyone: Group::new("", "Everyone", AccessRights((st_mode & 0b111) as u8)),
                attributes: Attributes::default(),
            })
        }

        #[cfg(target_os = "windows")]
        unsafe {
            let (user, admin, everyone) = win32::get_file_perms(value)?;
            Ok(Self {
                user,
                group: admin,
                everyone,
                attributes: Attributes::from(value),
            })
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AccessRights(u8);
bitflags::bitflags! {
    impl AccessRights: u8 {
        const Read = 1<< 2;
        const Write = 1 << 1;
        const Execute = 1;
    }
}
impl AccessRights {
    pub fn readable(&self) -> bool {
        self.contains(Self::Read)
    }
    pub fn writable(&self) -> bool {
        self.contains(Self::Write)
    }
    pub fn executable(&self) -> bool {
        self.contains(Self::Execute)
    }
}
impl std::fmt::Display for AccessRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.readable().mode_char('r'),
            self.writable().mode_char('w'),
            self.executable().mode_char('x')
        )
    }
}
impl std::fmt::Debug for AccessRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccessRights")
            .field("read", &self.readable())
            .field("write", &self.writable())
            .field("execute", &self.executable())
            .finish()
    }
}
#[cfg(target_os = "windows")]
impl From<u32> for AccessRights {
    fn from(value: u32) -> Self {
        use windows::Win32::Storage::FileSystem::{
            FILE_ACCESS_RIGHTS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
        };
        let value = FILE_ACCESS_RIGHTS(value);
        let mut result = Self::empty();
        if value.contains(FILE_GENERIC_READ) {
            result |= Self::Read;
        }
        if value.contains(FILE_GENERIC_WRITE) {
            result |= Self::Write;
        }
        if value.contains(FILE_GENERIC_EXECUTE) {
            result |= Self::Execute;
        }
        result
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub domain: String,
    pub name: String,
    pub permissions: AccessRights,
}
impl User {
    pub fn readable(&self) -> bool {
        self.permissions.readable()
    }
    pub fn writable(&self) -> bool {
        self.permissions.writable()
    }
    pub fn executable(&self) -> bool {
        self.permissions.executable()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub domain: String,
    pub name: String,
    pub permissions: AccessRights,
}
impl Group {
    pub fn new<S: ToString, S2: ToString>(
        domain: S,
        name: S2,
        permissions: AccessRights,
    ) -> Self {
        Self {
            domain: domain.to_string(),
            name: name.to_string(),
            permissions,
        }
    }

    pub fn readable(&self) -> bool {
        self.permissions.readable()
    }
    pub fn writable(&self) -> bool {
        self.permissions.writable()
    }
    pub fn executable(&self) -> bool {
        self.permissions.executable()
    }
}

#[cfg(target_os = "windows")]
mod win32 {
    use std::{ffi::c_void, fmt::Debug, os::windows::ffi::OsStrExt, path::Path};

    use windows::{
        core::{Error, HRESULT, PCWSTR, PWSTR},
        Win32::{
            Foundation::{
                CloseHandle, LocalFree, BOOL, ERROR_ACCESS_DENIED, ERROR_INSUFFICIENT_BUFFER,
                HANDLE, HLOCAL,
            },
            Security::{
                AccessCheck,
                Authorization::{GetNamedSecurityInfoW, SE_FILE_OBJECT},
                CreateWellKnownSid, DuplicateToken, GetAce, GetTokenInformation, LookupAccountSidW,
                MapGenericMask, SecurityImpersonation, TokenUser, WinBuiltinAdministratorsSid,
                WinWorldSid, ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, DACL_SECURITY_INFORMATION,
                GENERIC_MAPPING, GROUP_SECURITY_INFORMATION, OWNER_SECURITY_INFORMATION,
                PRIVILEGE_SET, PSECURITY_DESCRIPTOR, PSID, SID, SID_NAME_USE, TOKEN_DUPLICATE,
                TOKEN_IMPERSONATE, TOKEN_READ, TOKEN_USER, WELL_KNOWN_SID_TYPE,
            },
            Storage::FileSystem::{
                FILE_ACCESS_RIGHTS, FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ,
                FILE_GENERIC_WRITE,
            },
            System::Threading::{GetCurrentProcess, OpenProcessToken},
        },
    };

    use super::{AccessRights, Group, User};

    macro_rules! pvoid {
        (* mut $value: expr) => {
            std::ptr::addr_of_mut!($value) as *mut c_void
        };
        (* $value: expr) => {
            std::ptr::addr_of!($value) as *const c_void
        };
        (mut $value: expr) => {
            $value as *mut _ as *mut c_void
        };
        ($value: expr) => {
            $value as *const _ as *const c_void
        };
        (mut [] $value: expr) => {
            $value.as_mut_ptr() as *mut c_void
        };
        ([] $value: expr) => {
            $value.as_ptr() as *const c_void
        };
    }

    trait AsSIDPtr {
        fn into_sid_ptr(self) -> PSID;
    }
    impl<T> AsSIDPtr for &mut Vec<T> {
        fn into_sid_ptr(self) -> PSID {
            PSID(pvoid!(mut []self))
        }
    }
    impl AsSIDPtr for &mut SID {
        fn into_sid_ptr(self) -> PSID {
            PSID(pvoid!(mut self))
        }
    }
    impl AsSIDPtr for *mut SID {
        fn into_sid_ptr(self) -> PSID {
            PSID(pvoid!(mut self))
        }
    }
    impl AsSIDPtr for PSID {
        fn into_sid_ptr(self) -> PSID {
            self
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum SidType {
        User,
        Group,
        Domain,
        Alias,
        WellKnownGroup,
        DeletedAccount,
        Invalid,
        Unknown,
        Computer,
        Label,
        LogonSession,
    }

    impl From<SID_NAME_USE> for SidType {
        fn from(value: SID_NAME_USE) -> Self {
            match value.0 {
                1 => Self::User,
                2 => Self::Group,
                3 => Self::Domain,
                4 => Self::Alias,
                5 => Self::WellKnownGroup,
                6 => Self::DeletedAccount,
                7 => Self::Invalid,
                8 => Self::Unknown,
                9 => Self::Computer,
                10 => Self::Label,
                11 => Self::LogonSession,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Clone, PartialEq, Eq)]
    pub struct SId {
        sid: SID,
        domain: String,
        name: String,
        sid_type: SidType,
        permissions: AccessRights,
    }

    impl Debug for SId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("SId")
                .field("domain", &self.domain)
                .field("name", &self.name)
                .field("sid_type", &self.sid_type)
                .field("permissions", &self.permissions)
                .finish()
        }
    }

    impl TryFrom<*mut SID> for SId {
        type Error = Box<dyn std::error::Error>;
        fn try_from(sid: *mut SID) -> Result<Self, Self::Error> {
            let (domain, name, sid_type) = unsafe { lookup_account(sid) }?;
            Ok(Self {
                sid: unsafe { *sid },
                domain,
                name,
                sid_type,
                permissions: AccessRights::empty(),
            })
        }
    }

    pub unsafe fn lookup_account(
        sid: *mut SID,
    ) -> Result<(String, String, SidType), Box<dyn std::error::Error>> {
        let mut name_cap = 0u32;
        let mut name: Vec<u16> = Vec::new();
        let mut domain_cap = 0u32;
        let mut domain: Vec<u16> = Vec::new();

        let mut name_use = SID_NAME_USE(0);
        match LookupAccountSidW(
            None,
            sid.into_sid_ptr(),
            PWSTR::from_raw(name.as_mut_ptr()),
            std::ptr::addr_of_mut!(name_cap),
            PWSTR::from_raw(domain.as_mut_ptr()),
            std::ptr::addr_of_mut!(domain_cap),
            std::ptr::addr_of_mut!(name_use),
        ) {
            Err(err) if err.code() == HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) => {
                name.resize(name_cap as usize, 0);
                domain.resize(domain_cap as usize, 0);
                LookupAccountSidW(
                    None,
                    sid.into_sid_ptr(),
                    PWSTR::from_raw(name.as_mut_ptr()),
                    std::ptr::addr_of_mut!(name_cap),
                    PWSTR::from_raw(domain.as_mut_ptr()),
                    std::ptr::addr_of_mut!(domain_cap),
                    std::ptr::addr_of_mut!(name_use),
                )?
            }
            Err(err) => return Err(err.into()),
            _ => return Err("Unexpected".into()),
        }

        Ok((
            String::from_utf16(
                &domain[..domain.iter().position(|v| *v == 0).unwrap_or(domain.len())],
            )?,
            String::from_utf16(&name[..name.iter().position(|v| *v == 0).unwrap_or(name.len())])?,
            SidType::from(name_use),
        ))
    }

    struct DeferDrop<F: FnMut()>(F);
    impl<F> Drop for DeferDrop<F>
    where
        F: FnMut(),
    {
        fn drop(&mut self) {
            (self.0)()
        }
    }

    impl TryFrom<(TOKEN_USER, AccessRights)> for User {
        type Error = Box<dyn std::error::Error>;
        fn try_from((user, rights): (TOKEN_USER, AccessRights)) -> Result<Self, Self::Error> {
            let sid = user.User.Sid.0 as *mut SID;
            let (domain, name, _) = unsafe { lookup_account(sid) }?;
            Ok(Self {
                domain,
                name,
                permissions: rights,
            })
        }
    }
    impl TryFrom<(SID, AccessRights)> for Group {
        type Error = Box<dyn std::error::Error>;
        fn try_from((mut sid, rights): (SID, AccessRights)) -> Result<Self, Self::Error> {
            let (domain, name, _) = unsafe { lookup_account(std::ptr::addr_of_mut!(sid)) }?;
            Ok(Self {
                domain,
                name,
                permissions: rights,
            })
        }
    }

    unsafe fn create_well_known(
        well_known: WELL_KNOWN_SID_TYPE,
    ) -> Result<SID, Box<dyn std::error::Error>> {
        let mut admin_sid: Vec<u8> = Vec::new();
        let mut used = 0u32;
        match CreateWellKnownSid(
            well_known,
            None,
            admin_sid.into_sid_ptr(),
            std::ptr::addr_of_mut!(used),
        ) {
            Err(err) if err.code() == HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) => {
                admin_sid.resize(used as usize, 0);
                CreateWellKnownSid(
                    well_known,
                    None,
                    admin_sid.into_sid_ptr(),
                    std::ptr::addr_of_mut!(used),
                )?;
            }
            _ => return Err("Unexpected".into()),
        }
        Ok(*(admin_sid.as_mut_ptr() as *mut SID))
    }

    unsafe fn get_user(
        security: PSECURITY_DESCRIPTOR,
        mask: FILE_ACCESS_RIGHTS,
    ) -> Result<User, Box<dyn std::error::Error>> {
        // Spoof as current user
        let mut handle = HANDLE::default();
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_IMPERSONATE | TOKEN_DUPLICATE | TOKEN_READ,
            std::ptr::addr_of_mut!(handle),
        )?;
        let mut imp_token = HANDLE::default();
        DuplicateToken(
            handle,
            SecurityImpersonation,
            std::ptr::addr_of_mut!(imp_token),
        )?;

        let mut buff: Vec<u8> = Vec::new();
        let mut size = 0u32;
        // Get size needed for token user
        match GetTokenInformation(
            imp_token,
            TokenUser,
            Some(buff.as_mut_ptr() as *mut _),
            size,
            std::ptr::addr_of_mut!(size),
        ) {
            Err(err) if err.code() == HRESULT::from_win32(ERROR_INSUFFICIENT_BUFFER.0) => {
                buff.resize(size as usize, 0);
                // Get token user
                GetTokenInformation(
                    imp_token,
                    TokenUser,
                    Some(buff.as_mut_ptr() as *mut _),
                    size,
                    std::ptr::addr_of_mut!(size),
                )?;
            }
            Err(err) => return Err(err.into()),
            _ => return Err("Expected user data".into()),
        }

        // Maps FILE_ACCESS_RIGHTS to the rights retrieved in AccessCheck
        let gm = GENERIC_MAPPING {
            GenericAll: FILE_ALL_ACCESS.0,
            GenericRead: FILE_GENERIC_READ.0,
            GenericWrite: FILE_GENERIC_WRITE.0,
            GenericExecute: FILE_GENERIC_EXECUTE.0,
        };
        let mut ps = PRIVILEGE_SET::default();
        let mut mask = mask.0;

        // Map desired rights to their generic mappings
        MapGenericMask(std::ptr::addr_of_mut!(mask), std::ptr::addr_of!(gm));

        // Rigths retrieved from access check
        let mut ar = 0u32;
        let mut len = size_of::<PRIVILEGE_SET>() as u32;
        // Status of whether access is granted to check for the rights
        let mut status = BOOL::default();

        AccessCheck(
            security,
            imp_token,
            mask,
            std::ptr::addr_of!(gm),
            Some(std::ptr::addr_of_mut!(ps)),
            std::ptr::addr_of_mut!(len),
            std::ptr::addr_of_mut!(ar),
            std::ptr::addr_of_mut!(status),
        )?;
        CloseHandle(imp_token)?;

        // Check for access denied. If so then continue with default rights (no rights).
        // Otherwise return the error from calling access check.
        if status.0 == 0 {
            let error = Error::from_win32();
            if error.code() != HRESULT::from_win32(ERROR_ACCESS_DENIED.0) {
                return Err(error.into());
            }
        }

        (
            *(buff.as_mut_ptr() as *mut TOKEN_USER),
            AccessRights::from(ar),
        )
            .try_into()
    }

    unsafe fn get_groups(acl: *const ACL) -> Result<(Group, Group), Box<dyn std::error::Error>> {
        let everyone_sid = create_well_known(WinWorldSid)?;
        let admin_sid = create_well_known(WinBuiltinAdministratorsSid)?;

        let mut everyone = Group::new("", "Everyone", AccessRights::empty());
        let mut admin = Group::new("BUILTIN", "Administrators", AccessRights::empty());

        let list: &ACL = &*acl;
        for i in 0..list.AceCount as u32 {
            let mut ace = std::ptr::null_mut();
            GetAce(acl, i, std::ptr::addr_of_mut!(ace))?;

            let header = &*(ace as *mut ACE_HEADER);
            if header.AceType == 0 {
                let allow = &mut *(ace as *mut ACCESS_ALLOWED_ACE);
                let sid = &mut allow.SidStart as *mut _ as *mut SID;

                if admin_sid == *sid {
                    admin.permissions |= AccessRights::from(allow.Mask);
                    continue;
                } else if everyone_sid == *sid {
                    everyone.permissions |= AccessRights::from(allow.Mask);
                    continue;
                }
            }
        }
        Ok((admin, everyone))
    }

    pub unsafe fn get_file_perms(
        file: impl AsRef<Path>,
    ) -> Result<(User, Group, Group), Box<dyn std::error::Error>> {
        let file_u16 = file
            .as_ref()
            .as_os_str()
            .encode_wide()
            .chain([0])
            .collect::<Vec<_>>();
        // Pointers that receive the output arguments
        let mut acl = std::ptr::null_mut();
        let mut group = PSID::default();
        let mut owner = PSID::default();
        let mut security_descriptor = PSECURITY_DESCRIPTOR::default();
        let err = GetNamedSecurityInfoW(
            PCWSTR::from_raw(file_u16.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION | GROUP_SECURITY_INFORMATION,
            Some(std::ptr::addr_of_mut!(owner)),
            Some(std::ptr::addr_of_mut!(group)),
            // Pass the *address* of the pointer (DACL)
            Some(std::ptr::addr_of_mut!(acl)),
            None,
            // Same here
            std::ptr::addr_of_mut!(security_descriptor),
        );
        if err.is_err() {
            // PERF: Log error
            //let error = Error::from(HRESULT::from_win32(err.0));
            return Ok((User::default(), Group::default(), Group::default()));
        }
        #[allow(unused_variables)]
        let sd_defer = DeferDrop(|| {
            LocalFree(HLOCAL(security_descriptor.0 as _));
        });

        let user = get_user(
            security_descriptor,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE | FILE_GENERIC_EXECUTE,
        )?;
        let (admin, everyone) = get_groups(acl)?;

        Ok((user, admin, everyone))
    }

    #[test]
    fn get_sd() {
        for entry in std::fs::read_dir("C:\\").unwrap() {
            let path = entry.unwrap().path();
            let path = match dunce::canonicalize(&path) {
                Err(_) => path,
                Ok(path) => path,
            };
            let (user, admin, everyone) = unsafe { get_file_perms(&path) }.unwrap();
            println!(
                "{}{}{}  {path:?}",
                user.permissions, admin.permissions, everyone.permissions,
            );
        }
    }
}
