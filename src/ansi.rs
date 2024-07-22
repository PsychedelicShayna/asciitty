pub enum Ansi {
    RESET,
    RED,
    GREEN,
    YELLOW,
    BLUE,
    MAGENTA,
    CYAN,
    INVALID,
}

impl Ansi {
    /// Returns the corresponding ANSI escape code for the enum value.
    /// The only exception is Ansi::INVALID, which returns an empty string.
    ///
    /// Example: Ansi::RED.code() => "\x1b[31m"
    pub fn code(&self) -> &str {
        match &self {
            Ansi::RESET => "\x1b[0m",
            Ansi::RED => "\x1b[31m",
            Ansi::GREEN => "\x1b[32m",
            Ansi::YELLOW => "\x1b[33m",
            Ansi::BLUE => "\x1b[34m",
            Ansi::MAGENTA => "\x1b[35m",
            Ansi::CYAN => "\x1b[36m",
            Ansi::INVALID => "",
        }
    }

    /// Converts a string to the corresponding ANSI enum.
    /// Unrecognized strings will return Ansi::INVALID
    ///
    /// Example: Ansi::from_string("red") => Ansi::RED
    pub fn from_str(&self, string: &str) -> Self {
        let string = string.to_uppercase();

        match string.as_str() {
            "RESET" => Ansi::RESET,
            "RED" => Ansi::RED,
            "GREEN" => Ansi::GREEN,
            "YELLOW" => Ansi::YELLOW,
            "BLUE" => Ansi::BLUE,
            "MAGENTA" => Ansi::MAGENTA,
            "CYAN" => Ansi::CYAN,
            _ => Ansi::INVALID,
        }
    }

    /// Strip any ANSI escape codes from a string.
    ///
    /// Example: Ansi::strip("\x1b[31mHello, world!\x1b[0m") => "Hello, world!"
    pub fn strip(string: &str) -> String {
        let mut in_ansi = false;

        string
            .chars()
            .filter(|c| match c {
                '\x1b' => {
                    in_ansi = true;
                    false
                }

                'm' if in_ansi => {
                    in_ansi = false;
                    false
                }

                _ => !in_ansi,
            })
            .collect()
    }

    /// Sandwiches a string between the ANSI escape codes this enum value represents.
    ///
    /// Example: Ansi::RED.apply("Hello, world!") => "\x1b[31mHello, world!\x1b[0m"
    pub fn apply(&self, string: &str) -> String {
        format!("{}{}{}", self.code(), string, Ansi::RESET.code())
    }

    /// Returns the true length of a string, ignoring ANSI escape codes.
    /// Abbreviated form of: Ansi::strip(string).len()
    ///
    /// Example: 
    ///     Ansi::RED.apply("Hello, world!").len()           => 28
    ///     Ansi::true_len(Ansi::RED.apply("Hello, world!")) => 13
    pub fn true_len(string: &str) -> usize {
        Ansi::strip(string).len()
    }
}
