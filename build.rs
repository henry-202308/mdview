fn main() {
    #[cfg(windows)]
    {
        let mut resource = winres::WindowsResource::new();
        resource.set_manifest_file("app.manifest");
        resource.set_icon("assets/mdview.ico");
        resource.set("FileDescription", "mdview - Markdown Reader");
        resource.set("ProductName", "mdview");
        resource.set("LegalCopyright", "Copyright (c) 2026 henry-202308");
        if let Err(err) = resource.compile() {
            println!("cargo:warning=Windows resource compilation failed: {err}");
        }
    }
}
