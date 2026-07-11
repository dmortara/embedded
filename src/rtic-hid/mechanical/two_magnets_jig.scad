/**
 * Copyright (c) 2026 Mortara Limited
 * Daniel Mortara <daniel.mortara@gmail.com>
 * 2026-07-10
 *
 * A simple jig to test the magnet sensor range.
 * A holow cylinder in the z axis holding the sensor
 * A holow bigger cylinder in the z axis which holds the magnets on the Y axis 
 * symetrically appart from the senso 
 */ 


render_part = "all";
//render_part = "magnet_carrier";
//render_part = "sensor_carrier";

// --- Sensor package dimensions (ALS31001LUAA) ---
sensor_z_sz = 4.0; // In mm
sensor_x_sz = 3.0;
sensor_y_sz = 1.5;

// --- magnets size (a cylinder)---
magnet_radius = 3.0; // mm
magnet_height = 3.0; // mm


// Sizes for the sensor carrier
sensor_carrier_inner_radius = sensor_x_sz/2.0;
sensor_carrier_outer_radius = sensor_x_sz/2.0+2.0;

// Size of magnet carrier
magnet_carrier_inner_radius = sensor_carrier_outer_radius+0.1;
magnet_carrier_outer_radius = sensor_carrier_outer_radius+0.1 + magnet_height + 2.0;
magnet_carrier_height = magnet_radius*2.0 + 4.0;

sensor_carrier_height = magnet_carrier_height+5.0;

$fn = 60;

/**
 * Sensor container (a small rod)
 */
module sensor_carrier() {
  difference() {
    cylinder(d=sensor_carrier_outer_radius*2.0, h= sensor_carrier_height);
    translate([0,0,-0.1])
    cylinder(d=sensor_carrier_inner_radius*2.0, h= sensor_carrier_height+0.2);
    //translate([0,0,2.0+sensor_z_sz])
    //cylinder(d=sensor_carrier_inner_radius*2.0+1, h= sensor_carrier_height+0.1);
  }
}

/**
 * magnets container (a small cilinder)
 * with a hole in the Z axis where the sensor will go
 * Another hole in the y axis orthogonal to the sensor hole to put two magnets
 */
module magnet_carrier() {
  difference() {
    // Shape here ...
    cylinder(d=magnet_carrier_outer_radius*2.0, h= magnet_carrier_height);
    translate([0, 0, -0.1]) // So the hole goes a bit beyon the botom of the magnet carrier
    cylinder(d=magnet_carrier_inner_radius*2.0, h= magnet_carrier_height+0.2);
    // ... minus all that follows after:
    // Magnets holes. 
    // Sequence is appyied in reverse order as with OpenGL
    translate([0, 0, magnet_radius+2.0])
    translate([0, magnet_carrier_outer_radius, 0])
    rotate([90,0,0])
    cylinder(d=magnet_radius*2.0,h=magnet_carrier_outer_radius*2.0);

    // Two sligly bigger holes to insert the magnet from the outside
    // with a small groom to kipt it in place
    groove = 0.2; // In mm for holding the manget
    for(i=[-1:2:1])
      translate([0, 0, magnet_radius+2.0])
      translate([0, i*(magnet_carrier_inner_radius+groove), 0])
      rotate([-i*90, 0, 0])
      cylinder(d=(magnet_radius+groove)*2.0,h=magnet_carrier_outer_radius - magnet_carrier_inner_radius );
  }
}

/**
 * Preview setup
 */
if (render_part == "sensor_carrier" || render_part == "all") {
  color("SteelBlue") sensor_carrier();
}
if (render_part == "magnet_carrier" || render_part == "all") {
  color("Coral") #magnet_carrier();
}

