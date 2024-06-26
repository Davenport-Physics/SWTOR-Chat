
use int_enum::IntEnum;

#[repr(i32)]
#[derive(IntEnum, Clone, Copy, PartialEq)]
pub enum SwtorChannel {
    SAY = 1,
    YELL = 2,
    EMOTE = 3,
    WHISPER = 4,
    PlayerAFK = 8,
    GLOBAL = 51,
    PVP   = 52,
    TRADE = 53,
    GROUP = 54,
    OP    = 55,
    OPSAnnouncement = 56,
    GUILD = 57,
    GuildOfficer = 58,
    CustomChannel = 60,
    OpsOfficer = 61,
    PlayerNotFound = 1003,
}