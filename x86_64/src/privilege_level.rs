pub enum PrivilegeLevel {
    // Ring0 (Most privilege). This level is used by critical system-software
    // components that require direct access to all processor and system resources.
    // Incl BIOS, memory management, and interrupt handlers.
    Ring0 = 0,
    // Ring1 (Moderate privilege). This level is used by less critical system-
    // software that can access a limited scope of system resources.
    // Used by Drivers. Access privileges are defined by the operating system
    Ring1 = 1,
    // Ring2 (Moderate privilege). Similar to Ring1, also defined by the OS
    Ring2 = 2,
    // Ring3 (Least privilege). This is used by applications. Most system
    // resources aren't allowed to be access directly, and must be requested
    // via the operating system
    Ring3 = 3,
}

impl PrivilegeLevel {
    pub fn from_u16(value: u16) -> PrivilegeLevel {
        match value {
            0 => PrivilegeLevel::Ring0,
            1 => PrivilegeLevel::Ring1,
            2 => PrivilegeLevel::Ring2,
            3 => PrivilegeLevel::Ring3,
            _ => panic!("{} is not a valid privilege level", value),
        }
    }
}
