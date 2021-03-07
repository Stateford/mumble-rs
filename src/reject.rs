enum RejectType {
    None = 0,
    WrongVersion = 1,
    InvalidUsername = 2,
    WrongUserPW = 3,
    WrongServerPW = 4,
    UsernameInUse = 5,
    ServerFull = 6,
    NoCertificate = 7,
    AuthenticatorFail = 8
}
struct RejectMessage {
    reject_type: Option<RejectType>,
    reason: Option<String>
}