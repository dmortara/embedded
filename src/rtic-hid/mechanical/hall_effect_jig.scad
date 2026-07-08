// ============================================================
//  ALS31001LUAA Hall Effect Sensor — Rotary Test Jig
//
//  Sensor : Allegro ALS31001LUAA  — 4.0 × 3.0 × 1.5 mm package
//  Magnet : 6 mm dia × 3 mm height, N52 neodymium cylinder
//
//  Parts (set render_part, then export each STL separately):
//    "carrier" — bare sensor pocket; drops into base from above
//    "base"    — plate with carrier pocket at front, post behind
//    "arm"     — rotates on post; at 0° magnet is above sensor
//    "all"     — all three side-by-side for preview
//
//  Layout concept (top view):
//
//       ← base_w →
//    ┌─────────────────┐   ─
//    │  ┌──────┐       │   ↑
//    │  │sensor│       │   │  gap_front + c_w
//    │  │carry │       │   │
//    │  └──────┘       │   │
//    │                 │   │
//    │       ●─── arm  │   │  arm_r (post behind sensor)
//    │      post       │   │
//    │                 │   ↓
//    └─────────────────┘   ─
//
//  At 0° (arm pointing toward front of base) the magnet pod is
//  directly above the sensor die.  Rotating ±θ sweeps the magnet
//  in an arc simulating joystick deflection.
//
//  Assembly:
//    1. Solder 30 AWG wires to sensor pads; thread through carrier slots.
//    2. Drop carrier into base pocket (sensor die face flush with base top).
//    3. Tighten M2 set screw in carrier side to clamp sensor.
//    4. Slide arm onto post (magnet pod faces down toward sensor).
//    5. Press magnet into arm pocket from below (press-fit).
//    6. Lock arm with M3 × 6 socket-head bolt through post top bore.
// ============================================================

render_part = "all";  // "carrier" | "base" | "arm" | "all"

// ---- Sensor package (ALS31001LUAA) ----
pkg_l  =  4.0;   // package length [mm]
pkg_w  =  3.0;   // package width  [mm]
pkg_h  =  1.5;   // package height [mm]  (die face is the top)

// ---- Magnet ----
mag_dia =  6.0;   // diameter [mm]
mag_h   =  3.0;   // height   [mm]

// ---- Air gap: sensor die face to magnet face [mm] ----
// 2 mm gives strong signal with 6 mm N52; increase to soften coupling.
die_gap =  2.0;

// ---- Pivot post offset behind sensor = arm radius [mm] ----
// Arm at 0° puts magnet directly above sensor.
// 20 mm gives reasonable linearity for ±30° simulated joystick travel.
arm_r   = 20.0;

// ---- Gap from base front edge to carrier front [mm] ----
gap_front = 6.0;

// ---- Tolerances ----
tol_slip  = 0.30;   // slip/rotating fit — arm bore on post
tol_press = 0.10;   // press fit — magnet into arm pocket
tol_pkt   = 0.20;   // drop-in fit — carrier into base pocket

// ---- Global ----
wall   = 2.5;
post_d = 5.0;
post_h = 10.0;
$fn    = 64;

// ============================================================
//  Derived sizes
// ============================================================
c_l   = pkg_l + 2*wall;   // carrier length  (9 mm)
c_w   = pkg_w + 2*wall;   // carrier width   (8 mm)
c_h   = pkg_h + wall;     // carrier height  (4 mm): floor + sensor body

hub_d = post_d + 2*wall;  // arm hub outer diameter
pod_d = mag_dia + 2*wall; // arm magnet pod outer diameter
arm_t = mag_h + wall;     // arm total thickness (die_gap sets geometry,
                           // arm bottom sits on base top so die_gap is
                           // achieved by making base_h = c_h and arm
                           // resting on base with magnet flush at bottom)

// Carrier position in base
carrier_ox = 0;            // centred later via base_w
carrier_oy = gap_front;    // front edge of carrier pocket

// Sensor die centre Y in base coords
sensor_cy  = carrier_oy + c_w/2;

// Post position: arm_r behind sensor
post_cy    = sensor_cy + arm_r;

// Base dimensions — wide enough for ±60° arm sweep
base_w = 2 * (arm_r * sin(60) + pod_d/2 + wall);
base_d = post_cy + arm_r * (1 - cos(60)) + pod_d/2 + wall;
base_h = c_h;   // base plate height equals carrier height → tops flush

// Re-derive carrier X (centred in base)
c_ox = (base_w - c_l) / 2;
post_cx = base_w / 2;

// ============================================================
//  PART 1 — Sensor Carrier
//
//  Drop-in from above into the base pocket.
//  Sensor die face is flush with the carrier top face.
//  M2 set screw from one short side clamps the component.
//  Wire exit slots on both long sides let pre-soldered wires exit.
// ============================================================
module sensor_carrier() {
    difference() {
        cube([c_l, c_w, c_h]);

        // Sensor pocket — opens from top, die face flush with carrier top
        translate([(c_l - pkg_l)/2 - tol_pkt,
                   (c_w - pkg_w)/2 - tol_pkt,
                   wall - 0.01])
            cube([pkg_l + 2*tol_pkt,
                  pkg_w + 2*tol_pkt,
                  pkg_h + 0.02]);

        // Bottom opening — exposes pads for wire soldering / probe access
        translate([(c_l - pkg_l)/2 + 1,
                   (c_w - pkg_w)/2 + 1,
                   -0.01])
            cube([pkg_l - 2, pkg_w - 2, wall + 0.02]);

        // Wire exit slots on both long (±X) faces (1.2 mm wide)
        for (xf = [0, c_l - 1.2])
            translate([xf,
                       (c_w - pkg_w)/2 - tol_pkt - 0.01,
                       wall + pkg_h - 1.2])
                cube([1.2, pkg_w + 2*tol_pkt + 0.02, 1.3]);

        // M2 set screw from front short side (−Y face)
        translate([c_l/2, -0.01, c_h - pkg_h/2 - 0.5])
            rotate([-90, 0, 0])
                cylinder(d = 2.2, h = wall + 0.02);
    }
}

// ============================================================
//  PART 2 — Base
//
//  Carrier pocket is at the front.
//  Pivot post is arm_r behind the sensor die centre.
//  Wire channel in the base floor lets leads exit to the front.
// ============================================================
module base() {
    difference() {
        cube([base_w, base_d, base_h]);

        // Carrier pocket (drop-in, tol_pkt clearance per side)
        translate([c_ox - tol_pkt,
                   carrier_oy - tol_pkt,
                   -0.01])
            cube([c_l + 2*tol_pkt,
                  c_w + 2*tol_pkt,
                  c_h + 0.02]);

        // Wire channel in floor: from under carrier pocket to front face
        translate([c_ox + 1,
                   -0.01,
                   0])
            cube([c_l - 2, carrier_oy + tol_pkt + 0.5, 1.8]);

        // Post bore (through base height only; post is solid above)
        translate([post_cx, post_cy, -0.01])
            cylinder(d = post_d + tol_slip, h = base_h + 0.02);

        // M3 corner mounting holes
        for (x = [4, base_w - 4], y = [4, base_d - 4])
            translate([x, y, -0.01])
                cylinder(d = 3.4, h = base_h + 0.02);
    }

    // Pivot post — solid, printed as one piece with the base
    translate([post_cx, post_cy, base_h])
        difference() {
            cylinder(d = post_d, h = post_h);
            // M3 retention bolt hole (6 mm deep from top)
            translate([0, 0, post_h - 6])
                cylinder(d = 3.4, h = 6.1);
        }
}

// ============================================================
//  PART 3 — Magnet Arm
//
//  Hub at origin; magnet pod at (0, -arm_r, 0).
//  When placed on the post the −Y direction faces the sensor.
//  At 0° the magnet is directly above the die.
//  Angle grooves every 10° give visual deflection reference.
// ============================================================
module magnet_arm() {
    groove_inner = hub_d/2 + 1;
    groove_outer = arm_r - pod_d/2 - 1;
    groove_len   = groove_outer - groove_inner;

    difference() {
        // Body: tapered hull from hub to pod
        hull() {
            cylinder(d = hub_d, h = arm_t);
            translate([0, -arm_r, 0])
                cylinder(d = pod_d, h = arm_t);
        }

        // Post bore — slip fit for free rotation
        translate([0, 0, -0.01])
            cylinder(d = post_d + tol_slip, h = arm_t + 0.02);

        // Magnet pocket — press-fit, opens from arm bottom
        // Magnet face flush with arm bottom → distance to die = die_gap
        translate([0, -arm_r, -0.01])
            cylinder(d = mag_dia + tol_press, h = mag_h + 0.01);

        // M3 retention bolt clearance through hub top
        translate([0, 0, arm_t - 3.5])
            cylinder(d = 3.5, h = 4.1);

        // Angle reference grooves every 10° on top face (−Y = 0°)
        for (a = [-60 : 10 : 60])
            rotate([0, 0, a])
                translate([-0.4, -groove_outer, arm_t - 0.7])
                    cube([0.8, groove_len, 0.8]);

        // Wider centre mark at 0° (arm pointing toward sensor)
        translate([-0.8, -groove_outer, arm_t - 0.7])
            cube([1.6, groove_len, 0.8]);
    }
}

// ============================================================
//  Preview layout
// ============================================================
sep = 10;

if (render_part == "carrier" || render_part == "all")
    color("SteelBlue") sensor_carrier();

if (render_part == "base" || render_part == "all")
    translate([c_l + sep, 0, 0])
        color("SlateGray") base();

if (render_part == "arm" || render_part == "all")
    translate([c_l + sep + base_w + sep, 0, 0])
        color("Coral") magnet_arm();
