/**
 * Supplemental type definitions for @xcodekit/xcode.
 *
 * These provide rich types for the parsed .pbxproj JSON structure
 * beyond what napi-rs auto-generates.
 */

/** ISA types for all known Xcode project object types. */
export type ISA =
  | "PBXBuildFile"
  | "PBXAppleScriptBuildPhase"
  | "PBXCopyFilesBuildPhase"
  | "PBXFrameworksBuildPhase"
  | "PBXHeadersBuildPhase"
  | "PBXResourcesBuildPhase"
  | "PBXShellScriptBuildPhase"
  | "PBXSourcesBuildPhase"
  | "PBXRezBuildPhase"
  | "PBXContainerItemProxy"
  | "PBXFileReference"
  | "PBXGroup"
  | "PBXVariantGroup"
  | "XCVersionGroup"
  | "PBXFileSystemSynchronizedRootGroup"
  | "PBXFileSystemSynchronizedBuildFileExceptionSet"
  | "PBXFileSystemSynchronizedGroupBuildPhaseMembershipExceptionSet"
  | "PBXNativeTarget"
  | "PBXAggregateTarget"
  | "PBXLegacyTarget"
  | "PBXProject"
  | "PBXTargetDependency"
  | "XCBuildConfiguration"
  | "XCConfigurationList"
  | "PBXBuildRule"
  | "PBXReferenceProxy"
  | "XCSwiftPackageProductDependency"
  | "XCRemoteSwiftPackageReference"
  | "XCLocalSwiftPackageReference";

/** A 24-character hexadecimal UUID string. */
export type UUID = string;

/** Boolean as 0 | 1 integer. */
export type BoolNumber = 0 | 1;

/** Boolean as YES/NO string. */
export type BoolString = "YES" | "NO" | "YES_ERROR" | "YES_AGGRESSIVE";

/** Source tree reference types. */
export type SourceTree = "BUILT_PRODUCTS_DIR" | "DEVELOPER_DIR" | "SOURCE_ROOT" | "SDKROOT" | "<group>" | "<absolute>";

/** CopyFilesBuildPhase destination subfolder spec. */
export enum SubFolder {
  absolutePath = 0,
  wrapper = 1,
  executables = 6,
  resources = 7,
  frameworks = 10,
  sharedFrameworks = 11,
  sharedSupport = 12,
  plugins = 13,
  javaResources = 15,
  productsDirectory = 16,
}

/** Container item proxy type. */
export enum ProxyType {
  targetReference = 1,
  reference = 2,
}

/** Common file type UTIs. */
export type FileType =
  | "sourcecode.swift"
  | "sourcecode.c.c"
  | "sourcecode.c.h"
  | "sourcecode.c.objc"
  | "sourcecode.cpp.cpp"
  | "sourcecode.cpp.objcpp"
  | "sourcecode.javascript"
  | "wrapper.application"
  | "wrapper.framework"
  | "wrapper.app-extension"
  | "wrapper.plug-in"
  | "wrapper.xcframework"
  | "compiled.mach-o.dylib"
  | "archive.ar"
  | "folder.assetcatalog"
  | "text.plist.xml"
  | "text.plist.strings"
  | "text.plist.entitlements"
  | "text.xcconfig"
  | "file.storyboard"
  | "file.xib"
  | "file.intentdefinition"
  | "image.png"
  | "image.jpeg"
  | "net.daringfireball.markdown"
  | string;

/** Common product type UTIs. */
export type ProductType =
  | "com.apple.product-type.application"
  | "com.apple.product-type.application.on-demand-install-capable"
  | "com.apple.product-type.app-extension"
  | "com.apple.product-type.bundle"
  | "com.apple.product-type.framework"
  | "com.apple.product-type.library.dynamic"
  | "com.apple.product-type.library.static"
  | "com.apple.product-type.tool"
  | "com.apple.product-type.unit-test-bundle"
  | "com.apple.product-type.ui-testing-bundle"
  | "com.apple.product-type.application.watchapp"
  | "com.apple.product-type.application.watchapp2"
  | "com.apple.product-type.watchkit-extension"
  | "com.apple.product-type.extensionkit-extension"
  | string;

/** Build settings dictionary. */
export interface BuildSettings {
  ALWAYS_SEARCH_USER_PATHS?: BoolString;
  ASSETCATALOG_COMPILER_APPICON_NAME?: string;
  CLANG_ENABLE_MODULES?: BoolString;
  CLANG_ENABLE_OBJC_ARC?: BoolString;
  CODE_SIGN_ENTITLEMENTS?: string;
  CODE_SIGN_IDENTITY?: string;
  CODE_SIGN_STYLE?: "Automatic" | "Manual";
  CURRENT_PROJECT_VERSION?: string;
  DEBUG_INFORMATION_FORMAT?: "dwarf" | "dwarf-with-dsym";
  DEVELOPMENT_TEAM?: string;
  GCC_OPTIMIZATION_LEVEL?: string;
  GCC_PREPROCESSOR_DEFINITIONS?: string | string[];
  GENERATE_INFOPLIST_FILE?: BoolString;
  INFOPLIST_FILE?: string;
  INFOPLIST_KEY_CFBundleDisplayName?: string;
  IPHONEOS_DEPLOYMENT_TARGET?: string;
  MACOSX_DEPLOYMENT_TARGET?: string;
  TVOS_DEPLOYMENT_TARGET?: string;
  WATCHOS_DEPLOYMENT_TARGET?: string;
  MARKETING_VERSION?: string;
  PRODUCT_BUNDLE_IDENTIFIER?: string;
  PRODUCT_NAME?: string;
  SWIFT_VERSION?: string;
  TARGETED_DEVICE_FAMILY?: string;
  [key: string]: string | string[] | number | undefined;
}

/** Base object with isa field. */
export interface PBXObjectBase {
  isa: ISA;
  [key: string]: any;
}

/** PBXBuildFile object. */
export interface PBXBuildFile extends PBXObjectBase {
  isa: "PBXBuildFile";
  fileRef?: UUID;
  productRef?: UUID;
  settings?: Record<string, any>;
  platformFilter?: string;
  platformFilters?: string[];
}

/** PBXFileReference object. */
export interface PBXFileReference extends PBXObjectBase {
  isa: "PBXFileReference";
  fileEncoding?: number;
  lastKnownFileType?: FileType;
  explicitFileType?: FileType;
  includeInIndex?: BoolNumber;
  name?: string;
  path?: string;
  sourceTree?: SourceTree;
}

/** PBXGroup object. */
export interface PBXGroup extends PBXObjectBase {
  isa: "PBXGroup";
  children: UUID[];
  name?: string;
  path?: string;
  sourceTree?: SourceTree;
}

/** Build phase (shared fields for all 8 types). */
export interface AbstractBuildPhase extends PBXObjectBase {
  buildActionMask?: number;
  files: UUID[];
  runOnlyForDeploymentPostprocessing?: BoolNumber;
}

/** PBXNativeTarget object. */
export interface PBXNativeTarget extends PBXObjectBase {
  isa: "PBXNativeTarget";
  buildConfigurationList: UUID;
  buildPhases: UUID[];
  buildRules: UUID[];
  dependencies: UUID[];
  name: string;
  productName?: string;
  productReference?: UUID;
  productType: ProductType;
  packageProductDependencies?: UUID[];
  fileSystemSynchronizedGroups?: UUID[];
}

/** PBXProject (root object). */
export interface PBXProject extends PBXObjectBase {
  isa: "PBXProject";
  buildConfigurationList: UUID;
  compatibilityVersion: string;
  developmentRegion: string;
  hasScannedForEncodings: BoolNumber;
  knownRegions: string[];
  mainGroup: UUID;
  productRefGroup?: UUID;
  projectDirPath: string;
  projectRoot: string;
  targets: UUID[];
  packageReferences?: UUID[];
  attributes?: {
    LastSwiftUpdateCheck?: string;
    LastUpgradeCheck?: string;
    TargetAttributes?: Record<UUID, Record<string, any>>;
    [key: string]: any;
  };
}

/** XCBuildConfiguration object. */
export interface XCBuildConfiguration extends PBXObjectBase {
  isa: "XCBuildConfiguration";
  name: string;
  buildSettings: BuildSettings;
  baseConfigurationReference?: UUID;
}

/** XCConfigurationList object. */
export interface XCConfigurationList extends PBXObjectBase {
  isa: "XCConfigurationList";
  buildConfigurations: UUID[];
  defaultConfigurationIsVisible?: BoolNumber;
  defaultConfigurationName?: string;
}

/** The top-level parsed .pbxproj structure. */
export interface ParsedProject {
  archiveVersion: number;
  classes: Record<string, any>;
  objectVersion: number;
  objects: Record<UUID, PBXObjectBase>;
  rootObject: UUID;
}
