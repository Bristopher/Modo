// SPDX-License-Identifier: AGPL-3.0-or-later

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_DESKTOP_AVAILABLE_ON_DESKTOP_AND_WEB_DESCRIPTOR = {
        key: "platform_support.desktop.available_on_desktop_and_web",
        message: "Available on your desktop, and on the web",
        comment: "Body copy in platform availability and download support copy. Keep wording clear about desktop, web, and mobile status.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_DESKTOP_DOWNLOAD_DESKTOP_INTRO_DESCRIPTOR = {
        key: "platform_support.desktop.download_desktop_intro",
        message: "Download {product_name} for {windows}, {macos}, and {linux}. Mobile apps are underway.",
        comment: "Download-page meta description and intro copy. Preserve {product_name} exactly; keep platform names conventional and make desktop/mobile availability clear. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_DESKTOP_INTERFACE_LABEL_DESCRIPTOR = {
        key: "platform_support.desktop.interface_label",
        message: "{product_name} desktop interface",
        comment: "Alt text for a desktop product screenshot. Preserve {product_name} exactly; describe the image as the desktop interface, not as a download action. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_DESKTOP_LABEL_DESCRIPTOR = {
        key: "platform_support.desktop.label",
        message: "Desktop",
        comment: "Short UI label or heading in platform availability and download support copy. Keep wording clear about desktop, web, and mobile status.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_DESKTOP_USE_DESKTOP_CLIENT_MOBILE_SOON_DESCRIPTOR = {
        key: "platform_support.desktop.use_desktop_client_mobile_soon",
        message: "Use the desktop client (mobile coming soon)",
        comment: "Body copy in platform availability and download support copy. Keep wording clear about desktop, web, and mobile status.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_DESKTOP_WINDOWS_GAME_CAPTURE_WARNING_TITLE_DESCRIPTOR = {
        key: "platform_support.desktop.windows_game_capture_warning.title",
        message: "{windows} {game_capture} build",
        comment: "Short heading for a callout on the download page about the optional Windows Game Capture build. Preserve Windows Game Capture as a feature name; keep it short. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_DESKTOP_WINDOWS_GAME_CAPTURE_WARNING_BODY_DESCRIPTOR = {
        key: "platform_support.desktop.windows_game_capture_warning.body",
        message: "The {game_capture} build is not code-signed yet, so {microsoft_defender} may quarantine it until {microsoft} approves it. The standard {windows} build is unaffected.",
        comment: "Body for the Windows Game Capture callout on the download page. Preserve Windows Game Capture and Microsoft Defender as names; make clear only the Game Capture build is affected and the standard build is fine. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_DONE_DESKTOP_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.done_desktop",
        message: "Done! You can now open {product_name} as if it were a regular program.",
        comment: "Body copy for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_DONE_MOBILE_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.done_mobile",
        message: "Done! You can now open {product_name} from your home screen.",
        comment: "Body copy for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_IN_CHROME_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.in_chrome",
        message: " in {chrome}",
        comment: "Compact UI label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_IN_CHROME_OR_ANOTHER_BROWSER_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.in_chrome_or_another_browser",
        message: " in {chrome} or another browser with PWA support",
        comment: "Body copy for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_IN_SAFARI_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.in_safari",
        message: " in Safari",
        comment: "Compact UI label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_PWA_INSTALLATION_GUIDE_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.pwa_installation_guide",
        message: "PWA installation guide for {name}",
        comment: "Compact UI label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_STEPS_PRESS_ADD_TO_HOME_SCREEN_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.steps.press_add_to_home_screen",
        message: "Press \"Add to home screen\"",
        comment: "Compact UI label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_STEPS_PRESS_ADD_UPPER_RIGHT_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.steps.press_add_upper_right",
        message: "Press \"Add\" in the upper-right corner",
        comment: "Body copy for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_STEPS_PRESS_INSTALL_APP_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.steps.press_install_app",
        message: "Press \"Install app\"",
        comment: "Compact UI label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_STEPS_PRESS_INSTALL_BUTTON_ADDRESS_BAR_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.steps.press_install_button_address_bar",
        message: "Press the install button (downward-pointing arrow on monitor) in the address bar",
        comment: "Button or link label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_STEPS_PRESS_INSTALL_IN_POPUP_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.steps.press_install_in_popup",
        message: "Press \"Install\" in the popup that appears",
        comment: "Body copy for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_STEPS_PRESS_MORE_MENU_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.steps.press_more_menu",
        message: "Press the \"More\" (⋮) button in the top-right corner",
        comment: "Body copy for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_GUIDES_STEPS_PRESS_SHARE_BUTTON_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.guides.steps.press_share_button",
        message: "Press the share button (rectangle with upward-pointing arrow)",
        comment: "Button or link label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_INSTALL_FLUXER_AS_APP_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.install_fluxer_as_app",
        message: "Install {product_name} as an app",
        comment: "Compact UI label for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INSTALL_AS_APP_TITLE_DESCRIPTOR = {
        key: "platform_support.mobile.install_as_app.title",
        message: "How to install as an app",
        comment: "Short UI label or heading for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_INTERFACE_LABEL_DESCRIPTOR = {
        key: "platform_support.mobile.interface_label",
        message: "{product_name} mobile interface",
        comment: "Short UI label or heading for mobile/PWA install guidance on the download page. Keep instructions clear, device-appropriate, and concise; preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_ANDROID_APK_DESCRIPTOR = {
        key: "platform_support.platforms.android.apk",
        message: "APK",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_ANDROID_MIN_VERSION_DESCRIPTOR = {
        key: "platform_support.platforms.android.min_version",
        message: "{android} 8+",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_ANDROID_NAME_DESCRIPTOR = {
        key: "platform_support.platforms.android.name",
        message: "{android}",
        comment: "Short UI label or heading naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_IOS_IOS_IPADOS_DESCRIPTOR = {
        key: "platform_support.platforms.ios.ios_ipados",
        message: "{ios} and {ipados}",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_IOS_MIN_VERSION_DESCRIPTOR = {
        key: "platform_support.platforms.ios.min_version",
        message: "{ios} 15+",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_IOS_NAME_DESCRIPTOR = {
        key: "platform_support.platforms.ios.name",
        message: "{ios}",
        comment: "Short UI label or heading naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_IOS_TESTFLIGHT_DESCRIPTOR = {
        key: "platform_support.platforms.ios.testflight",
        message: "{testflight}",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_LINUX_CHOOSE_DISTRIBUTION_DESCRIPTOR = {
        key: "platform_support.platforms.linux.choose_distribution",
        message: "Choose {linux} distribution",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_LINUX_NAME_DESCRIPTOR = {
        key: "platform_support.platforms.linux.name",
        message: "{linux}",
        comment: "Short UI label or heading naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_LINUX_RECOMMENDED_DESCRIPTOR = {
        key: "platform_support.platforms.linux.recommended",
        message: "recommended",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_MACOS_APPLE_SILICON_DESCRIPTOR = {
        key: "platform_support.platforms.macos.apple_silicon",
        message: "{apple_silicon}",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_MACOS_DOWNLOAD_LABEL_DESCRIPTOR = {
        key: "platform_support.platforms.macos.download_label",
        message: "Download for {macos}",
        comment: "Button or link label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_MACOS_INTEL_DESCRIPTOR = {
        key: "platform_support.platforms.macos.intel",
        message: "Intel",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_MACOS_MIN_VERSION_DESCRIPTOR = {
        key: "platform_support.platforms.macos.min_version",
        message: "{macos} 10.15+",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_MACOS_NAME_DESCRIPTOR = {
        key: "platform_support.platforms.macos.name",
        message: "{macos}",
        comment: "Short UI label or heading naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_WINDOWS_DOWNLOAD_LABEL_DESCRIPTOR = {
        key: "platform_support.platforms.windows.download_label",
        message: "Download for {windows}",
        comment: "Button or link label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_WINDOWS_GAME_CAPTURE_BUILD_DESCRIPTOR = {
        key: "platform_support.platforms.windows.game_capture_build",
        message: "{game_capture}",
        comment: "Compact label for a Windows desktop build variant that includes the game capture module. Keep it short and preserve the Game Capture feature name. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_WINDOWS_MIN_VERSION_DESCRIPTOR = {
        key: "platform_support.platforms.windows.min_version",
        message: "{windows} 10+",
        comment: "Compact UI label naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_WINDOWS_NAME_DESCRIPTOR = {
        key: "platform_support.platforms.windows.name",
        message: "{windows}",
        comment: "Short UI label or heading naming a platform, installer, architecture, or minimum version in download UI. Keep platform names conventional and labels compact. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_WINDOWS_STANDARD_BUILD_DESCRIPTOR = {
        key: "platform_support.platforms.windows.standard_build",
        message: "Standard",
        comment: "Compact label for the default Windows desktop build variant without the game capture module. Keep it short for download menus.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_PLATFORMS_PORTABLE_DESCRIPTOR = {
        key: "platform_support.platforms.portable",
        message: "Portable",
        comment: "Compact UI label for a portable (no-install) desktop build that stores all data next to the executable. Keep it short.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_ON_MOBILE_TITLE_DESCRIPTOR = {
        key: "platform_support.mobile.on_mobile.title",
        message: "{product_name} on the go",
        comment: "Heading for the mobile section on the download page. Preserve {product_name} exactly. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_ON_MOBILE_INTRO_DESCRIPTOR = {
        key: "platform_support.mobile.on_mobile.intro",
        message: "Three ways to use {product_name} on your phone right now.",
        comment: "Intro copy under the mobile section heading on the download page. Preserve {product_name} exactly; keep it short. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_WEB_APP_TITLE_DESCRIPTOR = {
        key: "platform_support.mobile.web_app.title",
        message: "Web app",
        comment: "Card title for the mobile web app (PWA) option on the download page. Keep it short.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_WEB_APP_BODY_DESCRIPTOR = {
        key: "platform_support.mobile.web_app.body",
        message: "{product_name} runs in any mobile browser and installs to your home screen like a Progressive Web App. It is the most complete way to use {product_name} on a phone today.",
        comment: "Body copy for the mobile web app (PWA) card on the download page. Preserve {product_name} exactly; mention that it installs like a Progressive Web App. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_IOS_TITLE_DESCRIPTOR = {
        key: "platform_support.mobile.ios.title",
        message: "{ios} app",
        comment: "Card title for the iOS app option on the download page. Keep the platform name conventional and short. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_IOS_BODY_DESCRIPTOR = {
        key: "platform_support.mobile.ios.body",
        message: "As of 15 June 2026, the {ios} app is in a limited {testflight} beta with a small group of {premium_tier_name} subscribers. {testflight} slots are limited, so the best way in for most people is to wait for the public release.",
        comment: "Body copy for the iOS app card on the download page. Preserve {premium_tier_name} exactly; make clear the iOS app is in a limited TestFlight beta with capped slots and that a public release is coming. Do not promise that subscribing grants beta access. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_ANDROID_TITLE_DESCRIPTOR = {
        key: "platform_support.mobile.android.title",
        message: "{android} app",
        comment: "Card title for the Android app option on the download page. Keep the platform name conventional and short. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_ANDROID_BODY_DESCRIPTOR = {
        key: "platform_support.mobile.android.body",
        message: "Install the {android} APK straight from our open source repository on {github}.",
        comment: "Body copy for the Android app card on the download page. Keep APK and GitHub as proper names; make clear the install file lives in the open source repository. Preserve placeholders exactly.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_ANDROID_CTA_DESCRIPTOR = {
        key: "platform_support.mobile.android.cta",
        message: "Download the APK",
        comment: "Button or link label on the Android app card that opens the GitHub repository where the APK is published. Keep APK as a proper name; keep it short.",
    };
);

crate::marketing_message!(
    pub const PLATFORM_SUPPORT_MOBILE_ALPHA_DISCLAIMER_DESCRIPTOR = {
        key: "platform_support.mobile.alpha_disclaimer",
        message: "The {ios} and {android} apps are early alphas. They may be missing features you expect or still have bugs, and we are working hard to fix that. The web app is not an alpha, and it stays the most complete mobile option for now.",
        comment: "Disclaimer under the mobile section on the download page. Make clear the native iOS and Android apps are early alphas and may be buggy, while the web app (PWA) is not an alpha. Preserve placeholders exactly.",
    };
);
