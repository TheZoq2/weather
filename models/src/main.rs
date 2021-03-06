mod battery;
mod rainmeter;

#[macro_use]
extern crate scad;
use scad_util;
use scad::*;


use scad_util::compositions::object_at_corners;
use scad_util::constants::{x_axis, y_axis, z_axis};
use scad_util::shapes::cut_triangle;

qstruct!(Anemometer() {
    ball_radius: f32 = 20.,
    ball_thickness: f32 = 1.5,
    arm_length: f32 = 70.,
    arm_thickness: f32 = 3.,
    arm_height: f32 = 3.,
    l_mount_length: f32 = 8.,
    hub_hole_diameter: f32 = 3.5,
    screwhole_diameter: f32 = 3.,
    hub_radius: f32 = 10.,
    magnet_hole_offset: f32 = hub_radius/2.,
    sensor_depth: f32 = 1.,
});

impl Anemometer {
    fn arm(&self) -> ScadObject {
        let ball = {
            let outside = scad!(Sphere(Radius(self.ball_radius)));
            let bottom_cutout = {
                let cube = centered_cube(
                    vec3(self.ball_radius * 2., self.ball_radius*2., self.ball_radius),
                    (true, true, false)
                );
                scad!(Translate(vec3(0.,0.,-self.ball_radius)); cube)
            };
            let inner_cutout = scad!(Sphere(Radius(self.ball_radius - self.ball_thickness)));

            scad!(Difference; {
                outside,
                inner_cutout,
                bottom_cutout
            })
        };

        let arm_ball_offset = self.ball_radius-self.ball_thickness;

        let arm = {
            let shape = centered_cube(
                vec3(self.arm_length, self.arm_thickness, self.arm_height),
                (false, true, false)
            );
            scad!(Translate(vec3(arm_ball_offset, 0., 0.)); shape)
        };

        let l_mount = {
            let shape = scad!(Cube(
                vec3(self.arm_thickness, self.l_mount_length, self.arm_thickness),
            ));
            let x_offset = arm_ball_offset + self.arm_length - self.arm_thickness;
            scad!(Translate(vec3(x_offset, 0., 0.)); shape)
        };

        scad!(Union; {
            ball,
            arm,
            l_mount
        })
    }

    fn hub(&self) -> ScadObject {
        let radius = self.hub_radius;
        let height = self.l_mount_length + 2.;
        let arm_start_radius = 5.;
        let arm_hole_padding = 0.5;
        let magnet_diameter = 5.;
        let magnet_padding = 0.2;
        let magnet_thickness = 1.;

        let main = scad!(Cylinder(height, Radius(radius)));

        let screw_hole = scad!(Cylinder(height, Diameter(self.hub_hole_diameter)));

        let arm_hole = {
            let thickness = self.arm_thickness + arm_hole_padding;
            let vertical_section = centered_cube(
                vec3(thickness, thickness, height),
                (true, true, false)
            );
            let horizontal_section = centered_cube(
                vec3(radius, thickness, thickness),
                (false, true, false)
            );

            scad!(Translate(vec3(arm_start_radius, 0., 0.)); {
                scad!(Union; {
                    vertical_section,
                    horizontal_section
                })
            })
        };

        let arm_holes = (0..3).fold(scad!(Union), |mut acc, i| {
            let angle = (i as f32) * 360./3.;
            acc.add_child(scad!(Rotate(angle, vec3(0., 0., 1.)); {
                arm_hole.clone()
            }));
            acc
        });

        let magnet_hole = {
            let diameter = magnet_diameter + magnet_padding;
            let shape = scad!(Cylinder(magnet_thickness, Diameter(diameter)));

            let z_offset = height - magnet_thickness;
            let xy_offset = self.magnet_hole_offset;
            scad!(Translate(vec3(xy_offset, xy_offset, z_offset)); shape)
        };

        let magnet_holes = (0..3)
                .map(|r| r as f32)
                .fold(scad!(Union), |mut acc, r| {
                    acc.add_child(scad!(Rotate(r * 360./3., vec3(0.,0.,1.)); magnet_hole.clone()));
                    acc
                });

        scad!(Difference;
            scad!(Union; {
                main
            }),
            scad!(Union; {
                screw_hole,
                arm_holes,
                magnet_holes
            })
        )
    }

    pub fn hall_sensor_cutout(&self) -> ScadObject {
        let padding = 0.5;
        let sensor_size = vec3(4.0 + padding, 4., 1.5 + padding);
        let wire_hole_length = 5.;
        let thickness = 4.;

        let sensor_hole = centered_cube(sensor_size, (true, true, false));
        let wire_hole = {
            let shape = centered_cube(
                sensor_size + vec3(0.,wire_hole_length,thickness),
                (true, false, false)
            );
            scad!(Translate(vec3(0., sensor_size.y/2., -thickness)); {
                shape
            })
        };

        scad!(Translate(vec3(0., 0., -sensor_size.z)); {
            sensor_hole,
            wire_hole
        })
    }

    pub fn mount(&self, screwhole_length: f32) -> ScadObject {
        let screwhole = centered_cylinder(screwhole_length * 2., Diameter(self.screwhole_diameter));

        let sensor = scad!(Translate(x_axis() * self.hub_radius / 2.); {
            scad!(Rotate(-90., z_axis()); self.hall_sensor_cutout())
        });

        scad!(Union; {
            screwhole,
            sensor
        })
    }

    fn base(&self) -> ScadObject {
        let padding = 0.5;
        let thickness = 4.;
        let sensor_top_offset = 0.5;
        let nut_height = 2.5;
        let nut_width = 5.5 + padding;

        let cylinder_radius = 20.;
        let leg_length = 25.;

        let hall_sensor_cutout = self.hall_sensor_cutout();

        let translated_hall_sensor = {
            let xy = self.magnet_hole_offset;
            scad!(Translate(vec3(xy, xy, thickness - sensor_top_offset)); {
                hall_sensor_cutout
            })
        };

        let legs = {
            let leg_thickness = 5.;

            let base = scad!(Cylinder(leg_length, Radius(cylinder_radius)));
            let inner_cutout = scad!(Cylinder(leg_length, Radius(cylinder_radius - leg_thickness)));

            let leg_cutout = {
                let shape = centered_cube(
                    vec3(cylinder_radius*2. - leg_thickness*2., 100., leg_length),
                    (true, true, false)
                );
                shape
            };

            let shape = scad!(Difference; {
                base,
                inner_cutout,
                leg_cutout
            });

            let screwholes = {
                let shape = scad!(Rotate(90., vec3(0., 1., 0.)); {
                    centered_cylinder(100., Diameter(3.5))
                });

                scad!(Translate(vec3(0., 0., 7.)); shape)
            };

            scad!(Translate(vec3(0., 0., -leg_length)); {
                scad!(Difference; {
                    shape,
                    screwholes
                })
            })
        };

        let body = scad!(Union; {
            scad!(Cylinder(thickness, Radius(cylinder_radius))),
            legs
        });

        let screwhole = scad!(Cylinder(thickness, Diameter(self.hub_hole_diameter - 0.5)));
        scad!(Difference; {
            body,
            screwhole,
            translated_hall_sensor,
            scad_util::nut(nut_width, nut_height)
        })
    }
}

qstruct!(Housing() {
    pcb_padding: f32 = 4.,
    pcb_x_size: f32 = 70. +  pcb_padding,
    pcb_y_size: f32 = 25.,
    pcb_z_size: f32 = 90. + pcb_padding,
    wall_thickness: f32 = 4.,

    outer_x_size: f32 = pcb_x_size + wall_thickness,
    outer_z_size: f32 = pcb_z_size + wall_thickness,

    screwhead_diameter: f32 = 6.5,
    screwhole_thread_diameter: f32 = 3.,
    screwhole_diameter: f32 = 3.7,

    battery_mount_lower_len: f32 = 15.,
    battery_mount_upper_len: f32 = 20.,
    battery_mount_z_len: f32 = 5.,

    wire_hole_diameter: f32 = 15.,

    grid_mount_hole_padding: f32 = 6.,
});

impl Housing {
    fn assembly(&self) -> ScadObject {
        scad!(Union; {
            self.watertight_section(),
            scad!(Translate(vec3(0., 55., 0.)); self.water_seal()),
            scad!(Translate(vec3(0., 80., 0.)); self.sensor_section()),
            scad!(Translate(vec3(0., 90., 0.)); self.wire_hole_water_seal(2.)),
            scad!(Translate(vec3(0., 100., 0.)); self.wire_hole_cover()),
            scad!(Translate(vec3(0., 130., 0.)); self.grid()),
        })
    }

    fn outer_shape(&self, screwhole_diameter: f32, y_size: f32) -> ScadObject {
        // Componnents
        let outer = centered_cube(
            vec3(self.outer_x_size, y_size, self.outer_z_size),
            (true, false, true)
        );

        let outer_screwholes = {
            let outer_shape = centered_cube(
                vec3(self.screwhead_diameter, y_size, self.screwhead_diameter),
                (true, false, true)
            );
            let cutout = {
                let shape = scad!(Cylinder(y_size, Diameter(screwhole_diameter)));
                scad!(Rotate(-90., x_axis()); shape)
            };

            self.object_at_outer_screwholes(scad!(Difference; { outer_shape, cutout }))
        };

        scad!(Union; {
            outer,
            outer_screwholes
        })
    }

    fn watertight_section(&self) -> ScadObject {
        // Sizes
        let back_thickness = 6.;
        let pcb_screwhole_diameter = 3.;
        let pcb_screwhole_depth = back_thickness - 1.;
        let y_size = self.pcb_y_size + back_thickness;

        let mount_screwhole_diameter = 3.;
        let mount_screwhole_depth = back_thickness - 1.;
        let mount_screwhole_x_separation = 50.;
        let mount_screwhole_z_separation = 50.;

        let object_at_pcb_holes = |object: ScadObject| {
            let x_distance = 62.;
            let z_distance = 80.;

            object_at_corners(x_axis(), z_axis(), x_distance, z_distance, object)
        };

        let cutout = {
            let shape = centered_cube(
                vec3(self.pcb_x_size, self.pcb_y_size, self.pcb_z_size),
                (true, false, true)
            );
            scad!(Translate(vec3(0., back_thickness, 0.)); shape)
        };

        let pcb_screwholes = {
            let shape = scad!(Cylinder(pcb_screwhole_depth, Diameter(pcb_screwhole_diameter)));
            let rotated = scad!(Rotate(-90., vec3(1., 0., 0.)); shape);

            scad!(Translate(vec3(0., back_thickness - pcb_screwhole_depth, 0.)); object_at_pcb_holes(rotated))
        };

        let mount_screwholes = {
            let shape = scad!(Cylinder(
                mount_screwhole_depth,
                Diameter(mount_screwhole_diameter)
            ));
            object_at_corners(
                x_axis(),
                z_axis(),
                mount_screwhole_x_separation,
                mount_screwhole_z_separation,
                scad!(Rotate(-90., x_axis()); shape)
            )
        };

        let battery_mount = {
            let shape = cut_triangle(
                self.battery_mount_lower_len,
                self.battery_mount_upper_len,
                self.battery_mount_z_len,
                y_size
            );

            let offset = self.outer_z_size / 2.;

            scad!(Translate(vec3(0., 0., -offset)); scad!(Rotate(-90., x_axis()); shape))
        };

        scad!(Difference; {
            scad!(Union; {
                self.outer_shape(self.screwhole_thread_diameter, y_size),
                battery_mount
            }),
            cutout,
            pcb_screwholes,
            mount_screwholes
        })
    }

    fn object_at_outer_screwholes(&self, object: ScadObject) -> ScadObject {
        object_at_corners(
            x_axis(),
            z_axis(),
            self.outer_x_size + self.screwhead_diameter,
            self.outer_z_size - self.screwhead_diameter,
            object
        )
    }

    fn object_at_wire_hole_screwholes(&self, object: ScadObject) -> ScadObject {
        let padding = 5.;
        let distance = self.wire_hole_diameter + padding;

        object_at_corners(x_axis(), z_axis(), distance, distance, object)
    }

    pub fn water_seal(&self) -> ScadObject {
        let thickness = 2.;
        let height = 6.;

        let outer = self.outer_shape(self.screwhole_diameter, thickness);
        let outer_box = centered_cube(
            vec3(self.pcb_x_size, height, self.pcb_z_size),
            (true, false, true)
        );
        let cutout = centered_cube(
            vec3(self.pcb_x_size - thickness, height, self.pcb_z_size - thickness),
            (true, false, true)
        );

        scad!(Difference; {
            scad!(Union; {
                outer,
                outer_box
            }),
            cutout
        })
    }

    pub fn wire_hole_water_seal(&self, thickness: f32) -> ScadObject {
        let shape = {
            let corner_shape = scad!(Rotate(-90., x_axis()); {
                scad!(Cylinder(thickness, Diameter(self.screwhead_diameter)))
            });
            scad!(Hull; self.object_at_wire_hole_screwholes(corner_shape))
        };

        let screwholes = self.object_at_wire_hole_screwholes(
                scad!(Rotate(-90., x_axis()); {
                    scad!(Cylinder(thickness, Diameter(self.screwhole_diameter)))
                })
            );

        scad!(Difference; {
            shape,
            screwholes
        })
    }

    pub fn wire_hole_cover(&self) -> ScadObject {
        let thickness = 3.;
        scad!(Difference; {
            self.wire_hole_water_seal(thickness),
            centered_cube(
                vec3(self.wire_hole_diameter, thickness, self.wire_hole_diameter),
                (true, false, false)
            ),
        })
    }

    pub fn sensor_section(&self) -> ScadObject {
        let inner_y_size = 15.;
        let back_thickness = self.wall_thickness;
        let screwhole_height = 4.;
        let y_size = inner_y_size + screwhole_height;
        let chin_size = 8.;
        let battery_wire_hole_diameter = 5.;

        let outer = {
            let with_screwholes = self.outer_shape(self.screwhole_diameter, screwhole_height);
            let rest = centered_cube(
                vec3(self.outer_x_size, y_size - screwhole_height, self.outer_z_size),
                (true, false, true)
            );

            scad!(Union; {
                with_screwholes,
                rest,
            })
        };

        let wire_hole = {
            let main_cutout = centered_cube(
                vec3(self.wire_hole_diameter, back_thickness, self.wire_hole_diameter),
                (true, false, true)
            );

            let holes = self.object_at_wire_hole_screwholes(
                scad!(Rotate(-90., x_axis()); {
                    scad!(Cylinder(back_thickness, Diameter(self.screwhole_thread_diameter)))
                })
            );

            scad!(Union; {main_cutout, holes})
        };

        let cutout = {
            let shape = centered_cube(
                vec3(self.pcb_x_size, inner_y_size, self.outer_z_size - chin_size),
                (true, false, true)
            );

            scad!(Translate(vec3(0., back_thickness, 0.)); shape)
        };

        let grid_mounting_spots = self.object_at_grid_mounting_holes(
                centered_cube(
                    vec3(self.grid_mount_hole_padding, inner_y_size, self.grid_mount_hole_padding),
                    (true, false, true)
                )
            );

        let grid_mounting_holes = {
            let shape = scad!(Cylinder(inner_y_size, Diameter(self.screwhole_thread_diameter)));
            let rotated = scad!(Rotate(-90., x_axis()); shape);
            let translated = scad!(Translate(vec3(0., screwhole_height, 0.)); rotated);
            self.object_at_grid_mounting_holes(translated)
        };

        let anemometer_mount = {
            let anemometer = Anemometer::new();

            let y_offset = inner_y_size / 2. + back_thickness;
            let z_offset = self.outer_z_size / 2. - anemometer.sensor_depth;

            scad!(Translate(vec3(0., y_offset, z_offset)); anemometer.mount(chin_size))
        };

        let battery_hole = scad!(Translate(vec3(0., inner_y_size / 2., -self.outer_z_size / 2.)); {
            scad!(Cylinder(chin_size, Diameter(battery_wire_hole_diameter)))
        });

        scad!(Difference; {
            scad!(Union; {
                scad!(Difference; {
                    outer,
                    cutout,
                    wire_hole,
                    anemometer_mount,
                    battery_hole
                }),
                grid_mounting_spots
            }),
            grid_mounting_holes
        })
    }

    pub fn object_at_grid_mounting_holes(&self, object: ScadObject) -> ScadObject {
        let padding = 6.;

        object_at_corners(
            x_axis(),
            z_axis(),
            self.outer_x_size - padding,
            self.outer_z_size - padding,
            object
        )
    }

    pub fn grid(&self) -> ScadObject {
        let fin_amount = 10;
        let grid_thickness = 3.;
        let fin_length = 13.;
        let fin_separation = (self.outer_z_size - grid_thickness) / fin_amount as f32;

        let fin = {
            let shape = scad!(Polygon(PolygonParameters::new(vec!(
                vec2(0., 0.),
                vec2(fin_length, fin_length),
                vec2(fin_length + grid_thickness, fin_length),
                vec2(grid_thickness, 0.)
            ))));

            let extruded = scad!(
                LinearExtrude(
                    LinExtrudeParams {height: self.outer_x_size, center: true, ..Default::default()}
                ); {shape}
            );

            scad!(Rotate(90., y_axis()); extruded)
        };

        let offset = self.outer_z_size / 2.;
        let mut fins = scad!(Union);
        for i in 0..fin_amount {
            fins.add_child(scad!(Translate(-z_axis() * (fin_separation * i as f32 - self.outer_z_size/2.)); {
                fin.clone()
            }));
        }

        let sides = {
            let shape = centered_cube(
                vec3(self.wall_thickness, fin_length, self.outer_z_size),
                (true,false,true)
            );

            let mut result = scad!(Union);
            let wall_offset = self.outer_x_size / 2. - self.wall_thickness / 2.;
            for i in vec!(-1., 1.) {
                result.add_child(scad!(Translate(i * wall_offset * x_axis()); shape.clone()));
            }
            result
        };

        let grid_mounting_spots = self.object_at_grid_mounting_holes(
                centered_cube(
                    vec3(self.grid_mount_hole_padding, fin_length, self.grid_mount_hole_padding),
                    (true, false, true)
                )
            );

        let grid_mounting_holes = {
            let shape = scad!(Cylinder(fin_length, Diameter(self.screwhole_diameter)));
            let rotated = scad!(Rotate(-90., x_axis()); shape);
            self.object_at_grid_mounting_holes(rotated)
        };

        scad!(Difference; {
            scad!(Union; {
                sides,
                fins,
                grid_mounting_spots
            }),
            grid_mounting_holes
        })
    }
}

fn full_assembly() -> ScadObject {
    let housing = Housing::new();
    scad!(Union; {
        housing.watertight_section(),
        scad!(Translate(vec3(0., 55., 0.)); housing.water_seal()),
        scad!(Translate(vec3(0., 80., 0.)); housing.sensor_section()),
        scad!(Translate(vec3(0., 90., 0.)); housing.wire_hole_water_seal(2.)),
        scad!(Translate(vec3(0., 100., 0.)); housing.wire_hole_cover()),
        scad!(Translate(vec3(0., 0., -70.)); scad!(Rotate(90., x_axis()); wall_mount())),
        scad!(Translate(vec3(100., 0., -50.)); battery::Powerbank::new().container()),
        scad!(Translate(vec3(100., 0., 70.)); battery::Powerbank::new().lid())
    })
}

fn wall_mount() -> ScadObject {
    let housing = Housing::new();
    let screwhole_pad = {
        let screwhole_diameter = 4.;
        let outer_diameter = 7.;
        let separation = 50.;
        let thickness = 4.;

        let place_at_screwholes = |obj: ScadObject| {
            scad!(Union; {
                scad!(Translate(
                    x_axis() * (separation/2.) + outer_diameter / 2. * y_axis()
                ); obj.clone()),
                scad!(Translate(
                    -x_axis() * (separation/2.) + outer_diameter / 2. * y_axis()
                ); obj.clone())
            })
        };

        let outer = scad!(Hull; {
            place_at_screwholes(
                scad!(Cylinder(thickness, Diameter(outer_diameter)))
            )
        });

        scad!(Difference; {
            outer,
            place_at_screwholes(
                scad!(Cylinder(
                    thickness,
                    Diameter(screwhole_diameter)
                ))
            )
        })
    };

    let z_size = 23.;
    let extrude_params = LinExtrudeParams{height: z_size, .. Default::default()};
    let side_thickness = 5.;
    let padding = 0.5;

    let main_body = {
        let y_size = housing.battery_mount_z_len;
        let lower_length = housing.battery_mount_lower_len + side_thickness;
        let upper_length = housing.battery_mount_upper_len + side_thickness;

        let polygon = PolygonParameters::new(vec!(
                    vec2(upper_length / 2., 0.),
                    vec2(upper_length / 2., side_thickness - padding),
                    vec2(lower_length / 2., side_thickness + y_size),
                    vec2(-lower_length / 2., side_thickness + y_size),
                    vec2(-upper_length / 2., side_thickness - padding),
                    vec2(-upper_length / 2., 0.),
                ));

        scad!(LinearExtrude(extrude_params.clone()); {
            scad!(Polygon(polygon))
        })
    };

    let cutout = {
        let offset = side_thickness - padding;
        let y_size = housing.battery_mount_z_len + padding;
        let lower_length = housing.battery_mount_lower_len + padding * 2.;
        let upper_length = housing.battery_mount_upper_len + padding * 2.;


        let polygon = PolygonParameters::new(vec!(
                    vec2(upper_length / 2., offset + 0.),
                    vec2(lower_length / 2., offset + y_size),
                    vec2(-lower_length / 2., offset + y_size),
                    vec2(-upper_length / 2., offset + 0.),
                ));

        scad!(LinearExtrude(extrude_params.clone()); {
            scad!(Polygon(polygon))
        })
    };

    scad!(Difference; {
        scad!(Union; {
            main_body,
            screwhole_pad
        }),
        cutout
    })
}

fn save_file(filename: &str, object: ScadObject) {
    let mut sfile = ScadFile::new();

    sfile.set_detail(50);

    sfile.add_object(object);

    sfile.write_to_file(String::from(filename));
}


fn main() {
    let anemometer = Anemometer::new();
    save_file("arm.scad", anemometer.arm());
    save_file("hub.scad", anemometer.hub());
    save_file("anemometer_base.scad", anemometer.base());
    save_file("housingAssembly.scad", full_assembly());
    save_file("watertight_section.scad", Housing::new().watertight_section());
    save_file("waterSeal.scad", Housing::new().water_seal());
    save_file("wire_hole_water_seal.scad", Housing::new().wire_hole_water_seal(2.));
    save_file("wire_hole_cover.scad", Housing::new().wire_hole_cover());
    save_file("sensor_section.scad", Housing::new().sensor_section());
    save_file("grid_section.scad", Housing::new().grid());
    save_file("wallmount.scad", wall_mount());
    save_file("battery_box.scad", battery::Powerbank::new().container());
    save_file("battery_box_lid.scad", battery::Powerbank::new().lid());
    save_file("rainmeter_tray.scad", scad!( Union; {
            rainmeter::rainmeter_tray(),
            //rainmeter::rainmeter_mount()
        }
    ));
    save_file("rainmeter_mount.scad", rainmeter::rainmeter_mount());
}
