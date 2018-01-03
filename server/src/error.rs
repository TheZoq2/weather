error_chain! {
    foreign_links {
        Io(::std::io::Error);
        Json(::serde_json::Error);
        Toml(::toml::de::Error);
    }
}
