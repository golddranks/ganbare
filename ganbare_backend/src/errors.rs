    error_chain! {
        foreign_links {
            ::std::str::ParseBoolError, ParseBoolError;
            ::std::env::VarError, VarError;
            ::std::num::ParseIntError, ParseIntError;
            ::std::num::ParseFloatError, ParseFloatError;
            ::std::io::Error, StdIoError;
            ::diesel::result::Error, DieselError;
            ::diesel::migrations::RunMigrationsError, DieselMigrationError;
        }
        errors {
            InvalidInput {
                description("Provided input is invalid.")
                display("Provided input is invalid.")
            }
            NoSuchUser(email: String) {
                description("No such user exists")
                display("No user with e-mail address {} exists.", email)
            }
            EmailAddressTooLong {
                description("E-mail address too long")
                display("A valid e-mail address can be 254 characters at maximum.")
            }
            EmailAddressNotValid {
                description("E-mail address not valid")
                display("An e-mail address must contain the character '@'.")
            }
            PasswordTooShort {
                description("Password too short")
                display("A valid password must be at least 8 characters (bytes).")
            }
            PasswordTooLong {
                description("Password too long")
                display("A valid password must be at maximum 1024 characters (bytes).")
            }
            PasswordDoesntMatch {
                description("Password doesn't match")
                display("Password doesn't match.")
            }
            AuthError {
                description("Can't authenticate user")
                display("Username (= e-mail) or password doesn't match.")
            }
            BadSessId {
                description("Malformed session ID!")
                display("Malformed session ID!")
            }
            NoSuchSess {
                description("Session doesn't exist!")
                display("Session doesn't exist!")
            }
            FormParseError {
                description("Can't parse the HTTP form!")
                display("Can't parse the HTTP form!")
            }
            FileNotFound {
                description("Can't find that file!")
                display("Can't find that file!")
            }
            DatabaseOdd(reason: &'static str) {
                description("There's something wrong with the contents of the DB vs. how it should be!")
                display("There's something wrong with the contents of the DB vs. how it should be! {}", reason)
            }
            AccessDenied {
                description("Access denied")
                display("Access denied")
            }
            NoneResult {
                description("Option::None")
                display("Option::None")
            }
            RateLimitExceeded {
                description("RateLimit exceeded")
                display("RateLimit exceeded")
            }
        }
    }
