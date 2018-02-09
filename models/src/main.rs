#[macro_use]
extern crate scad_generator;
use scad_generator::*;


qstruct!(Anemometer() {
    ball_radius: f32 = 20.,
    ball_thickness: f32 = 1.5,
    arm_length: f32 = 70.,
    arm_thickness: f32 = 3.,
    arm_height: f32 = 3.,
    l_mount_length: f32 = 8.,
    hub_hole_diameter: f32 = 3.5,
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
        let radius = 10.;
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
            scad!(Translate(vec3(radius / 2., radius/2., z_offset)); shape)
        };

        scad!(Difference;
            scad!(Union; {
                main
            }),
            scad!(Union; {
                screw_hole,
                arm_holes,
                magnet_hole
            })
        )
    }
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
}
