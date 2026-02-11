use std::collections::HashMap;
use std::sync::LazyLock;

/// Maps file extensions to their Xcode `lastKnownFileType` UTI.
pub static FILE_TYPES_BY_EXTENSION: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("a", "archive.ar");
    m.insert("app", "wrapper.application");
    m.insert("appex", "wrapper.app-extension");
    m.insert("bundle", "wrapper.plug-in");
    m.insert("c", "sourcecode.c.c");
    m.insert("cc", "sourcecode.cpp.cpp");
    m.insert("cpp", "sourcecode.cpp.cpp");
    m.insert("css", "text.css");
    m.insert("cxx", "sourcecode.cpp.cpp");
    m.insert("d", "sourcecode.dtrace");
    m.insert("dylib", "compiled.mach-o.dylib");
    m.insert("entitlements", "text.plist.entitlements");
    m.insert("framework", "wrapper.framework");
    m.insert("gif", "image.gif");
    m.insert("gpx", "text.xml");
    m.insert("h", "sourcecode.c.h");
    m.insert("hh", "sourcecode.cpp.h");
    m.insert("hpp", "sourcecode.cpp.h");
    m.insert("html", "text.html");
    m.insert("hxx", "sourcecode.cpp.h");
    m.insert("ipp", "sourcecode.cpp.h");
    m.insert("intentdefinition", "file.intentdefinition");
    m.insert("jpeg", "image.jpeg");
    m.insert("jpg", "image.jpeg");
    m.insert("js", "sourcecode.javascript");
    m.insert("json", "text.json");
    m.insert("m", "sourcecode.c.objc");
    m.insert("markdown", "net.daringfireball.markdown");
    m.insert("md", "net.daringfireball.markdown");
    m.insert("mm", "sourcecode.cpp.objcpp");
    m.insert("modulemap", "sourcecode.module");
    m.insert("mp3", "audio.mp3");
    m.insert("pch", "sourcecode.c.h");
    m.insert("plist", "text.plist.xml");
    m.insert("png", "image.png");
    m.insert("s", "sourcecode.asm");
    m.insert("sh", "text.script.sh");
    m.insert("storyboard", "file.storyboard");
    m.insert("strings", "text.plist.strings");
    m.insert("stringsdict", "text.plist.stringsdict");
    m.insert("swift", "sourcecode.swift");
    m.insert("tbd", "sourcecode.text-based-dylib-definition");
    m.insert("ts", "sourcecode.javascript");
    m.insert("tsx", "sourcecode.javascript");
    m.insert("ttf", "file");
    m.insert("wav", "audio.wav");
    m.insert("xcassets", "folder.assetcatalog");
    m.insert("xcconfig", "text.xcconfig");
    m.insert("xcdatamodel", "wrapper.xcdatamodel");
    m.insert("xcdatamodeld", "wrapper.xcdatamodeld");
    m.insert("xcframework", "wrapper.xcframework");
    m.insert("xib", "file.xib");
    m.insert("xml", "text.xml");
    m.insert("yaml", "text.yaml");
    m.insert("yml", "text.yaml");
    m.insert("zip", "archive.zip");
    m
});

/// Maps product UTIs to file extensions.
pub static PRODUCT_UTI_EXTENSIONS: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("com.apple.product-type.application", "app");
    m.insert("com.apple.product-type.application.on-demand-install-capable", "app");
    m.insert("com.apple.product-type.app-extension", "appex");
    m.insert("com.apple.product-type.bundle", "bundle");
    m.insert("com.apple.product-type.framework", "framework");
    m.insert("com.apple.product-type.library.dynamic", "dylib");
    m.insert("com.apple.product-type.library.static", "a");
    m.insert("com.apple.product-type.tool", "");
    m.insert("com.apple.product-type.unit-test-bundle", "xctest");
    m.insert("com.apple.product-type.ui-testing-bundle", "xctest");
    m.insert("com.apple.product-type.application.watchapp", "app");
    m.insert("com.apple.product-type.application.watchapp2", "app");
    m.insert("com.apple.product-type.watchkit-extension", "appex");
    m
});

/// Maps file types to their default sourceTree values.
pub static SOURCETREE_BY_FILETYPE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("wrapper.application", "BUILT_PRODUCTS_DIR");
    m.insert("wrapper.framework", "BUILT_PRODUCTS_DIR");
    m.insert("compiled.mach-o.dylib", "BUILT_PRODUCTS_DIR");
    m.insert("wrapper.plug-in", "BUILT_PRODUCTS_DIR");
    m.insert("archive.ar", "BUILT_PRODUCTS_DIR");
    m
});

// SDK version constants (Xcode 26.2 / Dec 2025)
pub const LAST_KNOWN_IOS_SDK: &str = "26.2";
pub const LAST_KNOWN_OSX_SDK: &str = "26.2";
pub const LAST_KNOWN_TVOS_SDK: &str = "26.2";
pub const LAST_KNOWN_WATCHOS_SDK: &str = "26.2";
pub const LAST_KNOWN_VISIONOS_SDK: &str = "26.2";
pub const LAST_KNOWN_ARCHIVE_VERSION: i64 = 1;
pub const LAST_KNOWN_OBJECT_VERSION: i64 = 77;
pub const DEFAULT_OBJECT_VERSION: i64 = 46;
pub const LAST_UPGRADE_CHECK: &str = "2600";

/// Default build settings for new projects.
pub struct ProjectDefaultBuildSettings;

impl ProjectDefaultBuildSettings {
    pub fn all() -> HashMap<&'static str, &'static str> {
        let mut m = HashMap::new();
        m.insert("ALWAYS_SEARCH_USER_PATHS", "NO");
        m.insert("CLANG_ANALYZER_NONNULL", "YES");
        m.insert("CLANG_ANALYZER_NUMBER_OBJECT_CONVERSION", "YES_AGGRESSIVE");
        m.insert("CLANG_CXX_LANGUAGE_STANDARD", "gnu++14");
        m.insert("CLANG_CXX_LIBRARY", "libc++");
        m.insert("CLANG_ENABLE_MODULES", "YES");
        m.insert("CLANG_ENABLE_OBJC_ARC", "YES");
        m.insert("CLANG_ENABLE_OBJC_WEAK", "YES");
        m.insert("CLANG_WARN_BLOCK_CAPTURE_AUTORELEASING", "YES");
        m.insert("CLANG_WARN_BOOL_CONVERSION", "YES");
        m.insert("CLANG_WARN_COMMA", "YES");
        m.insert("CLANG_WARN_CONSTANT_CONVERSION", "YES");
        m.insert("CLANG_WARN_DEPRECATED_OBJC_IMPLEMENTATIONS", "YES");
        m.insert("CLANG_WARN_DIRECT_OBJC_ISA_USAGE", "YES_ERROR");
        m.insert("CLANG_WARN_DOCUMENTATION_COMMENTS", "YES");
        m.insert("CLANG_WARN_EMPTY_BODY", "YES");
        m.insert("CLANG_WARN_ENUM_CONVERSION", "YES");
        m.insert("CLANG_WARN_INFINITE_RECURSION", "YES");
        m.insert("CLANG_WARN_INT_CONVERSION", "YES");
        m.insert("CLANG_WARN_NON_LITERAL_NULL_CONVERSION", "YES");
        m.insert("CLANG_WARN_OBJC_IMPLICIT_RETAIN_SELF", "YES");
        m.insert("CLANG_WARN_OBJC_LITERAL_CONVERSION", "YES");
        m.insert("CLANG_WARN_OBJC_ROOT_CLASS", "YES_ERROR");
        m.insert("CLANG_WARN_QUOTED_INCLUDE_IN_FRAMEWORK_HEADER", "YES");
        m.insert("CLANG_WARN_RANGE_LOOP_ANALYSIS", "YES");
        m.insert("CLANG_WARN_STRICT_PROTOTYPES", "YES");
        m.insert("CLANG_WARN_SUSPICIOUS_MOVE", "YES");
        m.insert("CLANG_WARN_UNGUARDED_AVAILABILITY", "YES_AGGRESSIVE");
        m.insert("CLANG_WARN_UNREACHABLE_CODE", "YES");
        m.insert("CLANG_WARN__DUPLICATE_METHOD_MATCH", "YES");
        m.insert("COPY_PHASE_STRIP", "NO");
        m.insert("ENABLE_STRICT_OBJC_MSGSEND", "YES");
        m.insert("GCC_C_LANGUAGE_STANDARD", "gnu11");
        m.insert("GCC_NO_COMMON_BLOCKS", "YES");
        m.insert("GCC_WARN_64_TO_32_BIT_CONVERSION", "YES");
        m.insert("GCC_WARN_ABOUT_RETURN_TYPE", "YES_ERROR");
        m.insert("GCC_WARN_UNDECLARED_SELECTOR", "YES");
        m.insert("GCC_WARN_UNINITIALIZED_AUTOS", "YES_AGGRESSIVE");
        m.insert("GCC_WARN_UNUSED_FUNCTION", "YES");
        m.insert("GCC_WARN_UNUSED_VARIABLE", "YES");
        m.insert("MTL_ENABLE_DEBUG_INFO", "INCLUDE_SOURCE");
        m
    }

    pub fn debug() -> HashMap<&'static str, &'static str> {
        let mut m = HashMap::new();
        m.insert("DEBUG_INFORMATION_FORMAT", "dwarf");
        m.insert("ENABLE_TESTABILITY", "YES");
        m.insert("GCC_DYNAMIC_NO_PIC", "NO");
        m.insert("GCC_OPTIMIZATION_LEVEL", "0");
        m.insert("GCC_PREPROCESSOR_DEFINITIONS", "DEBUG=1 $(inherited)");
        m.insert("MTL_ENABLE_DEBUG_INFO", "INCLUDE_SOURCE");
        m.insert("ONLY_ACTIVE_ARCH", "YES");
        m
    }

    pub fn release() -> HashMap<&'static str, &'static str> {
        let mut m = HashMap::new();
        m.insert("DEBUG_INFORMATION_FORMAT", "dwarf-with-dsym");
        m.insert("ENABLE_NS_ASSERTIONS", "NO");
        m.insert("MTL_ENABLE_DEBUG_INFO", "NO");
        m.insert("VALIDATE_PRODUCT", "YES");
        m
    }
}
