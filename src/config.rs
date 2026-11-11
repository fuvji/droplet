//! Settings: one schema, INI persistence, and a byte-compatible migration from
//! the original `%TEMP%\settings.dat`.

use std::path::PathBuf;

pub trait Ini: Sized + Clone {
    fn ini_parse(s: &str) -> Option<Self>;
    fn ini_write(&self) -> String;
}

impl Ini for bool {
    fn ini_parse(s: &str) -> Option<Self> {
        let s = s.trim();
        if s == "1" || s.eq_ignore_ascii_case("true") {
            Some(true)
        } else if s == "0" || s.eq_ignore_ascii_case("false") {
            Some(false)
        } else {
            None
        }
    }
    fn ini_write(&self) -> String {
        if *self { "1".into() } else { "0".into() }
    }
}

impl Ini for i32 {
    fn ini_parse(s: &str) -> Option<Self> {
        s.trim().parse().ok()
    }
    fn ini_write(&self) -> String {
        self.to_string()
    }
}

impl Ini for String {
    fn ini_parse(s: &str) -> Option<Self> {
        Some(s.trim().to_string())
    }
    fn ini_write(&self) -> String {
        self.clone()
    }
}

macro_rules! settings_schema {
    ($( $field:ident : $key:literal : $ty:ty = $default:expr ; )*) => {
        #[derive(Clone)]
        pub struct Settings {
            $( pub $field: $ty, )*
        }

        impl Default for Settings {
            fn default() -> Self {
                Settings { $( $field: $default.into(), )* }
            }
        }

        impl Settings {
            pub fn set(&mut self, key: &str, val: &str) {
                match key {
                    $( $key => {
                        if let Some(v) = <$ty as Ini>::ini_parse(val) {
                            self.$field = v;
                        }
                    } )*
                    _ => {}
                }
            }

            pub fn get(&self, key: &str) -> String {
                match key {
                    $( $key => self.$field.ini_write(), )*
                    _ => String::new(),
                }
            }

            pub fn write_ini(&self) -> String {
                let mut out = String::with_capacity(2048);
                out.push_str("[T]\n");
                $( out.push_str($key); out.push('=');
                   out.push_str(&self.$field.ini_write()); out.push('\n'); )*
                out
            }

            pub fn read_ini(&mut self, text: &str) {
                for line in text.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('[') || line.starts_with(';') || line.starts_with('#') {
                        continue;
                    }
                    if let Some(eq) = line.find('=') {
                        let (k, v) = line.split_at(eq);
                        self.set(k.trim(), v[1..].trim());
                    }
                }
            }
        }
    };
}

settings_schema! {
    // ---- master + feature switches
    master:      "v0":  bool = true;
    dash:        "vD":  bool = false;
    shield:      "v1":  bool = false;
    mace:        "v2":  bool = false;
    breach:      "v3":  bool = false;
    potion:      "v4":  bool = true;
    anchor:      "v6":  bool = true;
    anchor_air:  "vE":  bool = true;
    crystal:     "v5":  bool = true;
    pearl:       "v7":  bool = true;
    catch_w:     "v9":  bool = true;
    totem:       "v8":  bool = true;
    fastuse:     "vA":  bool = true;
    sprint:      "vB":  bool = true;
    wtap:        "vC":  bool = true;
    stun:        "vF":  bool = true;
    auto_use:    "vW":  bool = false;
    ghost:       "vG":  bool = true;
    persist:     "vP":  bool = false;
    hover_inv:   "vI":  bool = true;
    low_latency: "vLL": bool = true;
    focus_only:  "vFO": bool = true;
    smart:       "vAI": bool = true;

    // ---- timing (milliseconds)
    // The three fast modules share a tight default profile: Min 47 ms and Max
    // 53 ms. The UI keeps Min inside 47-97 and Max inside 53-103 so a drag
    // cannot leave the useful range. Smart timing still varies inside the
    // selected envelope; manual mode uses it directly.
    crystal_min:  "v5D1": i32 = 47i32;
    crystal_max:  "v5D2": i32 = 53i32;
    anchor_min:   "v6D1": i32 = 47i32;
    anchor_max:   "v6D2": i32 = 53i32;
    wtap_min:     "vCD1": i32 = 132i32;
    wtap_max:     "vCD2": i32 = 148i32;
    dash_cd:      "vDCd": i32 = 1i32;
    mace_delay:   "v2D":  i32 = 1i32;
    fastuse_min:  "vAD1": i32 = 47i32;
    fastuse_max:  "vAD2": i32 = 53i32;
    // One-time migration marker for the fast-module default change. A config
    // written by this version carries vTR=1, so a deliberate 47/103 custom
    // range is never mistaken for the old default on the next launch.
    timing_rev:   "vTR": i32 = 0i32;
    ghost_off:    "vGMx": i32 = 2i32;
    win_x:        "winX": i32 = -1i32;
    win_y:        "winY": i32 = -1i32;

    // ---- modes / theme
    wtap_mode: "vM":   String = "S";
    theme:     "thmC": String = "E6E8EF";

    // ---- trigger modes (hold = fire on press, toggle = latch the repeat loop)
    dash_mode:    "vDMd": String = "hold";
    shield_mode:  "v1Md": String = "hold";
    mace_mode:    "v2Md": String = "hold";
    breach_mode:  "v3Md": String = "hold";
    potion_mode:  "v4Md": String = "hold";
    crystal_mode: "v5Md": String = "hold";
    anchor_mode:  "v6Md": String = "hold";
    pearl_mode:   "v7Md": String = "hold";
    totem_mode:   "v8Md": String = "hold";
    fastuse_mode: "vABMd": String = "hold";
    autouse_mode: "vWMd": String = "hold";

    // ---- trigger binds
    dash_key:     "vDBn": String = "b";
    dash_key2:    "vDB2": String = "...";
    shield_key:   "v1Bn": String = "q";
    shield_key2:  "v1B2": String = "...";
    mace_key:     "v2Bn": String = "r";
    mace_key2:    "v2B2": String = "...";
    breach_key:   "v3Bn": String = "XButton1";
    breach_key2:  "v3B2": String = "...";
    potion_key:   "v4Bn": String = "c";
    potion_key2:  "v4B2": String = "...";
    anchor_key:   "v6Bn": String = "x";
    anchor_key2:  "v6B2": String = "...";
    crystal_key:  "v5Bn": String = "z";
    crystal_key2: "v5B2": String = "...";
    fastuse_key:  "vABn": String = "XButton2";
    fastuse_key2: "vAB2": String = "...";
    sprint_key:   "vBBn": String = "y";
    sprint_key2:  "vBB2": String = "...";
    wtap_key:     "vCBn": String = "LButton";
    pearl_key:    "v7Bn": String = "MButton";
    pearl_key2:   "v7B2": String = "...";
    totem_key:    "v8Bn": String = "n";
    totem_key2:   "v8B2": String = "...";
    autouse_key:  "vWBn": String = "v";
    autouse_key2: "vWB2": String = "...";
    hover_inv_key:"v8I":  String = "e";

    // ---- action slots
    dash_spear:     "vDS1": String = "r";
    dash_restore:   "vDS2": String = "2";
    shield_axe:     "v1S1": String = "h";
    mace_slot:      "v2S1": String = "g";
    breach_mace:    "v3S1": String = "r";
    breach_restore: "v3S2": String = "1";
    potion_slot1:   "v4S1": String = "q";
    potion_slot2:   "v4S2": String = "f";
    potion_restore: "v4S3": String = "1";
    pearl_slot:     "v7S1": String = "q";
    pearl_catch:    "v7S3": String = "r";
    anchor_slot:    "v6S1": String = "h";
    glowstone_slot: "v6S2": String = "g";
    anchor_totem:   "v6S3": String = "r";
    crystal_obsidian: "v5S1": String = "XButton2";
    crystal_slot:   "v5S2": String = "3";
    totem_swap:     "v8Sp": String = "t";
    totem_hotbar:   "v8T":  String = "r";
    autouse_slot:   "vWS1": String = "4";
}

impl Settings {
    /// Hot UI path: read the boolean directly instead of formatting the value
    /// into a temporary String on every frame and every repaint.
    #[inline]
    pub fn bool_at(&self, key: &str) -> bool {
        match key {
            "v0" => self.master,
            "vD" => self.dash,
            "v1" => self.shield,
            "v2" => self.mace,
            "v3" => self.breach,
            "v4" => self.potion,
            "v6" => self.anchor,
            "vE" => self.anchor_air,
            "v5" => self.crystal,
            "v7" => self.pearl,
            "v9" => self.catch_w,
            "v8" => self.totem,
            "vA" => self.fastuse,
            "vB" => self.sprint,
            "vC" => self.wtap,
            "vF" => self.stun,
            "vW" => self.auto_use,
            "vG" => self.ghost,
            "vP" => self.persist,
            "vI" => self.hover_inv,
            "vLL" => self.low_latency,
            "vFO" => self.focus_only,
            "vAI" => self.smart,
            _ => false,
        }
    }

    /// Same allocation-free path for the numeric controls.
    #[inline]
    pub fn int_at(&self, key: &str) -> i32 {
        match key {
            "v5D1" => self.crystal_min,
            "v5D2" => self.crystal_max,
            "v6D1" => self.anchor_min,
            "v6D2" => self.anchor_max,
            "vCD1" => self.wtap_min,
            "vCD2" => self.wtap_max,
            "vDCd" => self.dash_cd,
            "v2D" => self.mace_delay,
            "vAD1" => self.fastuse_min,
            "vAD2" => self.fastuse_max,
            "vGMx" => self.ghost_off,
            "winX" => self.win_x,
            "winY" => self.win_y,
            _ => 0,
        }
    }
    pub fn str_at(&self, key: &str) -> String {
        self.get(key)
    }

    /// Borrowed lookup for every binding and slot chip in the panel. The paint
    /// path calls this once per chip per frame; returning `&str` keeps it
    /// allocation-free. Kept in sync with `get` by a release test.
    pub fn binding_at(&self, key: &str) -> &str {
        match key {
            "vDBn" => &self.dash_key,
            "vDB2" => &self.dash_key2,
            "vDS1" => &self.dash_spear,
            "vDS2" => &self.dash_restore,
            "v1Bn" => &self.shield_key,
            "v1B2" => &self.shield_key2,
            "v1S1" => &self.shield_axe,
            "v2Bn" => &self.mace_key,
            "v2B2" => &self.mace_key2,
            "v2S1" => &self.mace_slot,
            "v3Bn" => &self.breach_key,
            "v3B2" => &self.breach_key2,
            "v3S1" => &self.breach_mace,
            "v3S2" => &self.breach_restore,
            "v4Bn" => &self.potion_key,
            "v4B2" => &self.potion_key2,
            "v4S1" => &self.potion_slot1,
            "v4S2" => &self.potion_slot2,
            "v4S3" => &self.potion_restore,
            "v5Bn" => &self.crystal_key,
            "v5B2" => &self.crystal_key2,
            "v5S1" => &self.crystal_obsidian,
            "v5S2" => &self.crystal_slot,
            "v6Bn" => &self.anchor_key,
            "v6B2" => &self.anchor_key2,
            "v6S1" => &self.anchor_slot,
            "v6S2" => &self.glowstone_slot,
            "v6S3" => &self.anchor_totem,
            "v7Bn" => &self.pearl_key,
            "v7B2" => &self.pearl_key2,
            "v7S1" => &self.pearl_slot,
            "v7S3" => &self.pearl_catch,
            "v8Bn" => &self.totem_key,
            "v8B2" => &self.totem_key2,
            "v8Sp" => &self.totem_swap,
            "v8T" => &self.totem_hotbar,
            "v8I" => &self.hover_inv_key,
            "vABn" => &self.fastuse_key,
            "vAB2" => &self.fastuse_key2,
            "vWBn" => &self.autouse_key,
            "vWB2" => &self.autouse_key2,
            "vWS1" => &self.autouse_slot,
            "vBBn" => &self.sprint_key,
            "vBB2" => &self.sprint_key2,
            "vCBn" => &self.wtap_key,
            _ => "",
        }
    }
    pub fn set_bool(&mut self, key: &str, v: bool) {
        self.set(key, if v { "1" } else { "0" });
    }
    pub fn set_int(&mut self, key: &str, v: i32) {
        self.set(key, &v.to_string());
    }
    pub fn set_str(&mut self, key: &str, v: &str) {
        self.set(key, v);
    }

    /// Accent colour as (r, g, b); invalid values fall back to the default tint.
    pub fn theme_rgb(&self) -> (u8, u8, u8) {
        let hex = self.theme.trim_start_matches('#');
        if hex.len() >= 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return (r, g, b);
            }
        }
        (230, 232, 239)
    }

    pub fn set_theme_rgb(&mut self, r: i32, g: i32, b: i32) {
        self.theme = format!(
            "{:02X}{:02X}{:02X}",
            r.clamp(0, 255),
            g.clamp(0, 255),
            b.clamp(0, 255)
        );
    }

    /// Allocation-free mode lookup for the input hook and the paint path.
    pub fn mode_is_toggle(&self, key: &str) -> bool {
        match key {
            "vDMd" => self.dash_mode.eq_ignore_ascii_case("toggle"),
            "v1Md" => self.shield_mode.eq_ignore_ascii_case("toggle"),
            "v2Md" => self.mace_mode.eq_ignore_ascii_case("toggle"),
            "v3Md" => self.breach_mode.eq_ignore_ascii_case("toggle"),
            "v4Md" => self.potion_mode.eq_ignore_ascii_case("toggle"),
            "v5Md" => self.crystal_mode.eq_ignore_ascii_case("toggle"),
            "v6Md" => self.anchor_mode.eq_ignore_ascii_case("toggle"),
            "v7Md" => self.pearl_mode.eq_ignore_ascii_case("toggle"),
            "v8Md" => self.totem_mode.eq_ignore_ascii_case("toggle"),
            "vABMd" => self.fastuse_mode.eq_ignore_ascii_case("toggle"),
            "vWMd" => self.autouse_mode.eq_ignore_ascii_case("toggle"),
            _ => false,
        }
    }

    /// Dash cooldown is stored as a 3-stop index: 1 = 50 ms, 2 = 100 ms,
    /// 3 = 150 ms. The index keeps the config compact and the UI discrete.
    pub fn dash_cd_ms(&self) -> i32 {
        match self.dash_cd.clamp(1, 3) {
            1 => 50,
            2 => 100,
            _ => 150,
        }
    }

    /// Mace input lead is stored as the same compact 3-stop index as Dash.
    /// Stop 1 is the 50 ms default; stops 2 and 3 are 100/150 ms.
    pub fn mace_delay_ms(&self) -> i32 {
        match self.mace_delay.clamp(1, 3) {
            1 => 50,
            2 => 100,
            _ => 150,
        }
    }

    /// Keep older configs usable after the W-Tap mode rename and make the
    /// discrete cooldown robust against hand-edited INI files.
    pub fn normalize(&mut self) {
        if self.wtap_mode.eq_ignore_ascii_case("w") {
            self.wtap_mode = "W".into();
        } else {
            // S is the shipped default; the old "shield" spelling also lands
            // here, as do unknown values from hand-edited INI files.
            self.wtap_mode = "S".into();
        }
        if !(1..=3).contains(&self.dash_cd) {
            self.dash_cd = 1;
        }
        if !(1..=3).contains(&self.mace_delay) {
            self.mace_delay = 1;
        }
        for mode in [
            &mut self.dash_mode,
            &mut self.shield_mode,
            &mut self.mace_mode,
            &mut self.breach_mode,
            &mut self.potion_mode,
            &mut self.crystal_mode,
            &mut self.anchor_mode,
            &mut self.pearl_mode,
            &mut self.totem_mode,
            &mut self.fastuse_mode,
            &mut self.autouse_mode,
        ] {
            if mode.eq_ignore_ascii_case("toggle") {
                *mode = "toggle".into();
            } else {
                *mode = "hold".into();
            }
        }

        // Migrate the previous shipped defaults exactly once, without
        // touching a custom range on later launches. 47/103 was the 2.3
        // default for the two fast modules.
        if self.timing_rev < 1 {
            if self.crystal_min == 47 && self.crystal_max == 103 {
                self.crystal_max = 53;
            }
            if self.fastuse_min == 47 && self.fastuse_max == 103 {
                self.fastuse_max = 53;
            }
            self.timing_rev = 1;
        }
        // The first S-Tap release used W plus 100-120 ms. Move that exact old
        // default pair once; a user-customized mode or range is left alone.
        if self.timing_rev < 2 {
            if self.wtap_mode == "W" && self.wtap_min == 100 && self.wtap_max == 120 {
                self.wtap_mode = "S".into();
                self.wtap_min = 132;
                self.wtap_max = 148;
            }
            self.timing_rev = 2;
        }
        // Anchor now follows Hydrogen's 47/53 defaults. Migrate the old
        // 55/65 and 37/43 shipped pairs once; a deliberate custom range is
        // left alone.
        if self.timing_rev < 4 {
            if matches!(
                (self.anchor_min, self.anchor_max),
                (55, 65) | (37, 43) | (47, 103) | (37, 103)
            ) {
                self.anchor_min = 47;
                self.anchor_max = 53;
            }
            self.timing_rev = 4;
        }
        self.normalize_timing();
    }

    /// Enforce each slider's own domain without re-running legacy migrations.
    /// The two halves of a Min/Max pair are deliberately independent: editing
    /// one must never move the other. Callers that execute a pair sort it at
    /// read time (`smart::pick` already does this), so an inverted range is a
    /// valid temporary edit rather than a reason to rewrite the other slider.
    pub fn normalize_timing(&mut self) {
        self.crystal_min = self.crystal_min.clamp(47, 97);
        self.crystal_max = self.crystal_max.clamp(53, 103);
        // Hydrogen's Anchor sliders expose 47-97 and 53-103. Min and Max
        // remain independent; the pair is sorted when the macro reads it.
        self.anchor_min = self.anchor_min.clamp(47, 97);
        self.anchor_max = self.anchor_max.clamp(53, 103);
        self.fastuse_min = self.fastuse_min.clamp(47, 97);
        self.fastuse_max = self.fastuse_max.clamp(53, 103);
        self.wtap_min = self.wtap_min.clamp(50, 300);
        self.wtap_max = self.wtap_max.clamp(50, 300);
    }

}

pub fn data_dir_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let current = base.join("Droplet");
    let legacy = base.join("Hydrogen");
    // Keep existing users' bindings when the product is renamed. Only move the
    // directory when the new one does not exist; never overwrite a fresh config.
    if !current.exists() && legacy.exists() {
        let _ = std::fs::rename(&legacy, &current);
    }
    current
}

pub fn data_dir() -> PathBuf {
    let dir = data_dir_path();
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Remove only Droplet's own data directory (config, debug log, old helper
/// files). Never create it first: that would leave an empty trace behind.
pub fn purge_data() {
    let dir = data_dir_path();
    if dir.file_name().and_then(|n| n.to_str()) == Some("Droplet") {
        let _ = std::fs::remove_dir_all(dir);
    }
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.ini")
}

pub fn legacy_path() -> PathBuf {
    std::env::temp_dir().join("settings.dat")
}

impl Settings {
    pub fn load() -> Settings {
        let mut s = Settings::default();
        if let Ok(text) = std::fs::read_to_string(config_path()) {
            s.read_ini(&text);
            s.normalize();
            return s;
        }
        // One-time import of the AutoHotkey settings file (same key names).
        let legacy = legacy_path();
        if let Ok(text) = std::fs::read_to_string(&legacy) {
            s.read_ini(&text);
            // `vP` was inert in the original script (its default was 1 but nothing
            // read it), so keep the panel hidden until the user opts in.
            s.persist = false;
            s.normalize();
            s.save();
        }
        s.normalize();
        s
    }

    pub fn save(&self) {
        // Write beside the target and swap, so a crash or full disk cannot leave
        // a truncated config behind. Fall back to a direct write if the rename
        // is blocked (for example by a scanner holding the file).
        let path = config_path();
        let tmp = path.with_extension("tmp");
        let data = self.write_ini();
        if std::fs::write(&tmp, &data).is_ok() {
            if std::fs::rename(&tmp, &path).is_err() {
                let _ = std::fs::write(&path, &data);
                let _ = std::fs::remove_file(&tmp);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_timing_defaults_are_tight() {
        let s = Settings::default();
        assert_eq!((s.crystal_min, s.crystal_max), (47, 53));
        assert_eq!((s.anchor_min, s.anchor_max), (47, 53));
        assert_eq!((s.fastuse_min, s.fastuse_max), (47, 53));
        assert_eq!((s.wtap_min, s.wtap_max), (132, 148));
        assert_eq!(s.wtap_mode, "S");
    }

    #[test]
    fn old_wtap_defaults_migrate_once() {
        let mut s = Settings::default();
        s.timing_rev = 1;
        s.wtap_mode = "W".into();
        s.wtap_min = 100;
        s.wtap_max = 120;
        s.normalize();
        assert_eq!(s.wtap_mode, "S");
        assert_eq!((s.wtap_min, s.wtap_max), (132, 148));
        assert_eq!(s.timing_rev, 4);

        s.wtap_mode = "W".into();
        s.wtap_min = 100;
        s.wtap_max = 120;
        s.normalize();
        assert_eq!(s.wtap_mode, "W");
        assert_eq!((s.wtap_min, s.wtap_max), (100, 120));
    }

    #[test]
    fn previous_defaults_migrate_to_tight_profile() {
        let mut s = Settings::default();
        s.crystal_max = 103;
        s.anchor_min = 47;
        s.anchor_max = 53;
        s.fastuse_max = 103;
        s.normalize();
        assert_eq!((s.crystal_min, s.crystal_max), (47, 53));
        assert_eq!((s.anchor_min, s.anchor_max), (47, 53));
        assert_eq!((s.fastuse_min, s.fastuse_max), (47, 53));
        assert_eq!(s.timing_rev, 4);

        let mut old_anchor = Settings::default();
        old_anchor.anchor_min = 37;
        old_anchor.anchor_max = 43;
        old_anchor.normalize();
        assert_eq!((old_anchor.anchor_min, old_anchor.anchor_max), (47, 53));
    }

    #[test]
    fn migration_runs_once_and_preserves_a_later_custom_range() {
        let mut s = Settings::default();
        s.crystal_max = 103;
        s.normalize();
        assert_eq!(s.crystal_max, 53);
        assert_eq!(s.timing_rev, 4);

        s.crystal_max = 103;
        s.normalize();
        assert_eq!(s.crystal_max, 103);
    }

    #[test]
    fn timing_edits_are_independent_and_stay_inside_domains() {
        let mut s = Settings::default();

        // Moving Min must leave Max untouched, even when the two cross.
        s.crystal_min = 88;
        s.normalize_timing();
        assert_eq!((s.crystal_min, s.crystal_max), (88, 53));

        s.crystal_max = 103;
        s.normalize_timing();
        assert_eq!((s.crystal_min, s.crystal_max), (88, 103));

        // Anchor owns independent 47-97 and 53-103 domains.
        s.anchor_min = 90;
        s.normalize_timing();
        assert_eq!((s.anchor_min, s.anchor_max), (90, 53));
        s.anchor_max = 103;
        s.normalize_timing();
        assert_eq!((s.anchor_min, s.anchor_max), (90, 103));

        s.anchor_min = 999;
        s.anchor_max = 999;
        s.normalize_timing();
        assert_eq!((s.anchor_min, s.anchor_max), (97, 103));

        s.fastuse_min = 90;
        s.fastuse_max = 60;
        s.normalize_timing();
        assert_eq!((s.fastuse_min, s.fastuse_max), (90, 60));

        s.wtap_min = 300;
        s.wtap_max = 50;
        s.normalize_timing();
        assert_eq!((s.wtap_min, s.wtap_max), (300, 50));
    }

    #[test]
    fn runtime_normalization_preserves_a_custom_high_range() {
        let mut s = Settings::default();
        s.crystal_min = 47;
        s.crystal_max = 103;
        s.normalize_timing();
        assert_eq!(s.crystal_max, 103);
    }

    #[test]
    fn rgb_accent_round_trips_and_clamps_channels() {
        let mut s = Settings::default();
        assert_eq!(s.theme_rgb(), (230, 232, 239));
        s.set_theme_rgb(12, 240, 999);
        assert_eq!(s.theme, "0CF0FF");
        assert_eq!(s.theme_rgb(), (12, 240, 255));
        assert_eq!(s.get("thmC"), "0CF0FF");
    }

    #[test]
    fn fast_binding_lookup_covers_every_painted_chip() {
        let s = Settings::default();
        let keys = [
            "vDBn", "vDB2", "vDS1", "vDS2", "v1Bn", "v1B2", "v1S1", "v2Bn", "v2B2",
            "v2S1", "v3Bn", "v3B2", "v3S1", "v3S2", "v4Bn", "v4B2", "v4S1", "v4S2",
            "v4S3", "v5Bn", "v5B2", "v5S1", "v5S2", "v6Bn", "v6B2", "v6S1", "v6S2",
            "v6S3", "v7Bn", "v7B2", "v7S1", "v7S3", "v8Bn", "v8B2", "v8Sp", "v8T",
            "v8I", "vABn", "vAB2", "vWBn", "vWB2", "vWS1", "vBBn", "vBB2", "vCBn",
        ];
        for key in keys {
            assert_eq!(s.binding_at(key), s.get(key).as_str(), "{key}");
        }
        assert_eq!(s.binding_at("unknown"), "");
    }

    #[test]
    fn trigger_modes_default_to_hold_and_normalize_known_values() {
        let mut s = Settings::default();
        assert!(!s.mode_is_toggle("v1Md"));
        s.shield_mode = "TOGGLE".into();
        s.mace_mode = "spam".into();
        s.normalize();
        assert!(s.mode_is_toggle("v1Md"));
        assert!(!s.mode_is_toggle("v2Md"));
        assert_eq!(s.shield_mode, "toggle");
        assert_eq!(s.mace_mode, "hold");
    }
}
