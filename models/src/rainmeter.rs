const HEIGHT: f32 = 15.;
const LENGTH: f32 = 30.;
const MOUNT_DIAMETER: f32 = 5.;
const WALL_THICKNESS: f32 = 1.;
const INNER_WIDTH: f32 = 15.;
const WIDTH: f32 = INNER_WIDTH + WALL_THICKNESS * 2.;

use scad::*;

use scad_util::constants::{x_axis, y_axis, z_axis};

pub fn rainmeter_tray() -> ScadObject {
    let wall_thickness = WALL_THICKNESS;
    let length = LENGTH;
    let inner_width = INNER_WIDTH;
    let width = WIDTH;
    let height = HEIGHT;

    let outer_shape = {
        let points = vec![
            vec2(-length, wall_thickness),
            vec2(-length, 0.),
            vec2(length, 0.),
            vec2(length, wall_thickness),
            vec2(wall_thickness, wall_thickness / 2. + height),
            vec2(-wall_thickness, wall_thickness / 2. + height),
        ];

        let shape = scad!(Polygon(PolygonParameters::new(points)));

        let eparams = LinExtrudeParams{
            height: inner_width + wall_thickness * 2.,
            center: true,
            .. Default::default()
        };

        scad!(LinearExtrude(eparams); {shape})
    };

    let cutouts = {
        let base = centered_cube(vec3(length, height, inner_width), (false, false, true));

        let shape =
            scad!(Translate(vec3(wall_thickness/2., wall_thickness, 0.)); base);

        scad!(Union; {
            scad!(Mirror(x_axis()); {
                shape.clone()
            }),
            shape
        })
    };

    let mount_diameter = MOUNT_DIAMETER;
    let mount = {
        let shape = scad!(Cylinder(height/3., Diameter(mount_diameter)));

        let rotated = scad!(Rotate(-90., x_axis()); shape);
        scad!(Translate((width/2. + mount_diameter/3.) * z_axis()); {
            rotated
        })
    };

    scad!(Difference; {
        scad!(Union; {
            outer_shape,
            mount,
        }),
        cutouts,
    })
}

pub fn rainmeter_mount() -> ScadObject {
    let height = HEIGHT;
    let length = LENGTH;
    let screw_diameter = 4.;
    // 5. is very carefully calculated using a random number generated
    // let distance_from_base = (length*length) / (2.*height) - height - 5.;
    let distance_from_base = ((height/3.).powf(2.) + (length/3.).powf(2.)).sqrt() - 3. - WALL_THICKNESS;

    println!("{:#?}", distance_from_base);

    let mount_cutout = {
        let hole = scad!(Rotate(-90., x_axis()); {
            centered_cylinder(height/3.*2., Diameter(MOUNT_DIAMETER))
        });

        let phase = scad!(Translate(MOUNT_DIAMETER/3. * z_axis()); {
             centered_cube(vec3(100., 100., 100.), (true, false, false))
        });

        scad!(Union; {hole, phase})
    };


    let base_length = distance_from_base + screw_diameter + WALL_THICKNESS;
    // Double wall thickness
    let outer = {
        let shape = centered_cube(
            vec3(
                MOUNT_DIAMETER + WALL_THICKNESS*4.,
                base_length + height/3.,
                MOUNT_DIAMETER + WALL_THICKNESS * 4.
            ),
            (true, false, true)
        );

        scad!(Translate(-base_length * y_axis()); shape)
    };

    let screwhole_cutout = {
        let shape = centered_cylinder(100., Diameter(screw_diameter));
        
        scad!(Translate(-distance_from_base * y_axis()); shape)
    };

    scad!(Difference; {
        outer,
        mount_cutout,
        screwhole_cutout
    })
}
