use scad_generator::*;
use scad_util::constants::{x_axis, y_axis, z_axis};

qstruct!(Powerbank() {
    battery_height: f32 = 93.,
    wall_thickness: f32 = 1.5,
    lip_height: f32 = 5.,
    inner_x_size: f32 = 78.,
    inner_y_size: f32 = 30.,
    inner_z_size: f32 = battery_height + lip_height,
    bottom_thickness: f32 = 1.5,
});


impl Powerbank {
    pub fn place_bounding_cylinders(&self, object: ScadObject) -> ScadObject {
        let x_location = (self.inner_x_size - self.inner_y_size)/2. * x_axis();

        scad!(Union; {
            scad!(Translate(x_location); object.clone()),
            scad!(Translate(-x_location); object)
        })
    }

    pub fn container(&self) -> ScadObject {
        let outer = scad!(Hull; {
            self.place_bounding_cylinders(
                scad!(Cylinder(
                    self.inner_z_size + self.bottom_thickness,
                    Diameter(self.inner_y_size + self.wall_thickness * 2.)
                ))
            )
        });

        let inner = scad!(Translate(z_axis() * self.bottom_thickness); {
            scad!(Hull; {
                self.place_bounding_cylinders(
                    scad!(Cylinder(self.inner_z_size, Diameter(self.inner_y_size)))
                )
            })
        });

        let object_at_screwholes = |obj: ScadObject| {
            scad!(Union; {
                scad!(Translate(x_axis() * 85./2.); obj.clone()),
                scad!(Translate(-x_axis() * 85./2.); obj.clone())
            })
        };

        let screwbar = {
            let thickness = 4.;
            let screwhead_diameter = 7.;
            let screw_diameter = 4.;

            let outer_shape = scad!(Hull; {
                object_at_screwholes(scad!({
                    Cylinder(thickness, Diameter(screwhead_diameter))
                }))
            });

            let holes = object_at_screwholes(
                scad!(Cylinder(thickness, Diameter(screw_diameter)))
            );

            let shape = scad!(Difference; {
                outer_shape,
                holes
            });

            let rotated = scad!(Rotate(90., x_axis()); shape);
            scad!(Translate(vec3(
                0.,
                self.inner_y_size/2. + self.wall_thickness,
                self.inner_z_size - 10.
            )); rotated)
        };

        scad!(Difference; {
            scad!(Union; {
                outer,
                screwbar
            }),
            inner
        })
    }

    pub fn lid(&self) -> ScadObject {
        let padding = 0.5;
        let bottom = scad!(Hull; {
            self.place_bounding_cylinders(
                scad!(Cylinder(self.lip_height, Diameter(self.inner_y_size - padding)))
            )
        });

        let top = {
            let shape = scad!(Hull; {
                self.place_bounding_cylinders(
                    scad!(Cylinder(self.bottom_thickness, Diameter(self.inner_y_size + self.wall_thickness * 2.)))
                )
            });

            scad!(Translate(self.lip_height * z_axis()); shape)
        };


        let usb_x_offset = 22.5/2.;
        let usb_y_offset = 12. -25.5/2.;
        let usb_padding = 0.5;
        let usb_x_size = 12.;
        let usb_y_size = 4.5;

        let usb_port = scad!(Translate(vec3(usb_x_offset, usb_y_offset, 0.)); {
            scad!(Cube(vec3(
                usb_x_size + usb_padding,
                usb_y_size + usb_padding,
                self.bottom_thickness + self.lip_height,
            )))
        });

        scad!(Difference; {
            scad!(Union; {
                bottom,
                top
            }),
            usb_port
        })
    }
}
