# ============================================================
# Firefly Launcher - ProGuard / R8 Rules
# ============================================================

# ============================================================
# 1. Android Framework
# ============================================================
-keep public class * extends android.app.Activity
-keep public class * extends android.app.Service
-keep public class * extends android.content.BroadcastReceiver
-keep public class * extends android.content.ContentProvider
-keep public class * extends android.app.Application
-keep public class * extends android.preference.PreferenceActivity
-keep public class * extends android.app.Fragment
-keep public class * extends androidx.fragment.app.Fragment
-keep public class * extends androidx.preference.PreferenceFragmentCompat

# Keep enum values (required by Android)
-keepclassmembers enum * {
    public static **[] values();
    public static ** valueOf(java.lang.String);
}

# Keep custom Views (called by layout inflation)
-keep public class * extends android.view.View {
    public <init>(android.content.Context);
    public <init>(android.content.Context, android.util.AttributeSet);
    public <init>(android.content.Context, android.util.AttributeSet, int);
    public void set*(***);
}

# Keep onClick handlers in XML layouts
-keepclassmembers class * {
    public void *(android.view.View);
}

# Keep Parcelables
-keepclassmembers class * implements android.os.Parcelable {
    public static final ** CREATOR;
}

# ============================================================
# 2. JNI / Native Methods
# ============================================================
-keepclasseswithmembernames class * {
    native <methods>;
}

# ============================================================
# 3. Gson Serialization (@SerializedName and @Keep annotated)
# ============================================================
-keepattributes Signature
-keepattributes *Annotation*
-keep,allowobfuscation class com.google.gson.reflect.TypeToken
-keep,allowobfuscation class * extends com.google.gson.TypeAdapter
-keep class com.google.gson.** { *; }

# Keep all classes with @SerializedName (Gson)
-keep class com.externallogin.login.** { *; }

# Keep all @Keep annotated classes and members
-keep @androidx.annotation.Keep class * { *; }
-keepclassmembers @androidx.annotation.Keep class * { *; }

# JMinecraftVersionList and related (used for JSON parsing with Gson)
-keep class net.kdt.pojavlaunch.firefly.JMinecraftVersionList** { *; }
-keep class net.kdt.pojavlaunch.firefly.JAssets** { *; }
-keep class net.kdt.pojavlaunch.firefly.JAssetInfo** { *; }
-keep class net.kdt.pojavlaunch.firefly.value.** { *; }
-keep class net.kdt.pojavlaunch.firefly.value.DependentLibrary** { *; }
-keep class net.kdt.pojavlaunch.firefly.value.launcherprofiles.** { *; }

# ============================================================
# 4. Expression Builder (uses reflection)
# ============================================================
-keep class net.objecthunter.exp4j.ExpressionBuilder** { *; }

# ============================================================
# 5. Preference Screens (uses reflection for keys)
# ============================================================
-keep class net.kdt.pojavlaunch.firefly.prefs.screens** { *; }
-keep class net.kdt.pojavlaunch.firefly.prefs.LauncherPreferences** { *; }

# ============================================================
# 6. Custom Controls (serialized/deserialized via Gson)
# ============================================================
-keep class net.kdt.pojavlaunch.firefly.customcontrols.** { *; }

# ============================================================
# 7. Minecraft Account (serialized)
# ============================================================
-keep class net.kdt.pojavlaunch.firefly.value.MinecraftAccount** { *; }

# ============================================================
# 8. Logger (JNI)
# ============================================================
-keep class net.kdt.pojavlaunch.firefly.Logger { *; }

# ============================================================
# 9. EventBus
# ============================================================
-keepattributes *Annotation*
-keepclassmembers class * {
    @org.greenrobot.eventbus.Subscribe <methods>;
}
-keep enum org.greenrobot.eventbus.ThreadMode { *; }

# ============================================================
# 10. JavaGUILauncherActivity (EventBus subscriber)
# ============================================================
-keepclassmembers class net.kdt.pojavlaunch.firefly.JavaGUILauncherActivity {
    @org.greenrobot.eventbus.Subscribe <methods>;
}

# ============================================================
# 11. CallbackBridge (JNI + static methods called from native)
# ============================================================
-keep class org.lwjgl.glfw.CallbackBridge {
    public static <methods>;
}

# ============================================================
# 12. AWTInputBridge (JNI)
# ============================================================
-keep class net.kdt.pojavlaunch.firefly.AWTInputBridge {
    public static <methods>;
}

# ============================================================
# 13. VMLauncher (JNI)
# ============================================================
-keep class com.oracle.dalvik.VMLauncher {
    public static <methods>;
}

# ============================================================
# 14. JREUtils native methods
# ============================================================
-keepclassmembers class net.kdt.pojavlaunch.firefly.utils.JREUtils {
    public static native <methods>;
}

# ============================================================
# 15. Tools - fields accessed via reflection (version inheritance)
# ============================================================
-keepclassmembers class net.kdt.pojavlaunch.firefly.Tools {
    public static java.lang.String DIR_HOME_VERSION;
    public static java.lang.String DIR_HOME_LIBRARY;
}

# ============================================================
# 16. Version Info (JSON parsing)
# ============================================================
-keep class net.kdt.pojavlaunch.firefly.extra.** { *; }

# ============================================================
# 17. LauncherProfiles
# ============================================================
-keep class net.kdt.pojavlaunch.firefly.value.launcherprofiles.LauncherProfiles** { *; }

# ============================================================
# 18. Third-party libraries
# ============================================================

# OkHttp
-dontwarn okhttp3.**
-dontwarn okio.**
-keep class okhttp3.** { *; }
-keep interface okhttp3.** { *; }

# Apache Commons IO
-dontwarn org.apache.commons.**

# CommonMark
-dontwarn org.commonmark.**

# exp4j
-keep class net.objecthunter.exp4j.** { *; }

# HMCL VersionNumber
-keep class org.jackhuang.hmcl.util.versioning.VersionNumber { *; }

# HtmlCleaner
-dontwarn org.htmlcleaner.**

# ============================================================
# 19. StringFog (already handles string encryption)
# ============================================================
-keep class com.github.megatronking.stringfog.** { *; }

# ============================================================
# 20. Optimizations - maximize obfuscation
# ============================================================

# Remove debug info
-assumenosideeffects class android.util.Log {
    public static *** d(...);
    public static *** v(...);
    public static *** i(...);
}

# Don't warn about missing dependency classes
-dontwarn javax.annotation.**

# Aggressive optimization
-optimizationpasses 5
-overloadaggressively
-repackageclasses 'a'
-allowaccessmodification
-mergeinterfacesaggressively
