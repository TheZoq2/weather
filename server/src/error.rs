error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Json(::serde_json::Error);
        Toml(::toml::de::Error);
    }

    errors {
        NoSuchDataName(name: String) {
            description("no data with that name"),
            display("unknown data name: {}", name),
        }
        UnhandledURI(uri: String) {
            description("That uri is unkwnown"),
            display("Unhandled uri: {}", uri),
        }
    }
}
