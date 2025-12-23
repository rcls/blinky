
side = 7;

diamond_offset = 13.97;
square_offset = 11.938;

//diamond_center = diamond_offset + 7 / sqrt(2);
square_center = square_offset + 3.5;

slope = 30;
triangle = 55 / sqrt(3);

module board() {
    translate([-1,  1] * diamond_offset) face();
    translate([ 1, -1] * diamond_offset) face();
    translate([-1, -1] * diamond_offset) face();

    translate([1, 1] * square_offset) square(side);

    module face() {
        rotate(45) square(side, center=true);
    }
}

intersection() {
    union() {
        rotate([slope, 0, 0]) linear_extrude(100, center=true, convexity=2)
        rotate([-slope, 0,0]) board();

        rotate([slope, 0, 0]) difference() {
            translate([0, diamond_offset * sqrt(3) / 2, 0])
                cube([diamond_offset * 2, 2, 57], center=true);
            translate([1, 0, 11]) rotate([90, 0, 0]) intersection() {
                cylinder(r = 10, h=100, center=true);
                translate([0, -2, 0]) cube([22, 22, 100], center=true);
            }
        }

        rotate([slope, 0, 0]) {
            translate([square_center, 19, 1]) runner();
            translate([-diamond_offset, 19, 1]) runner();
            translate([square_center, 19, 26]) runner();
            translate([-diamond_offset, 19, 26]) runner();
        }

        rotate([slope, 0, 0]) translate([0, -diamond_offset * sqrt(3) / 2, 0])
        cube([diamond_offset * 2, 2, 30], center=true);

        difference() {
            union() {
                translate([-diamond_offset, 0, 0]) skew();
                translate([ diamond_offset, 0, 0]) skew();
            }
            intersection() {
                translate([0, -5, triangle/2-5]) rotate([0, 90, 0])
                    cylinder(r=7.5, h=60, center=true);
                translate([0, -5, triangle/2-6.5])
                    cube([60, 16, 16], center=true);
            }
        }

        translate([0, -diamond_offset, 0]) strut();
        translate([0,  diamond_offset, 0]) strut();
        translate([diamond_offset, 0, 0]) rotate(90) strut();
        translate([-diamond_offset, 0, 0]) rotate(90) strut();
    }

    difference() {
        translate([0, 0, triangle/2]) rotate([0, -90, 0])
            cylinder(r = triangle, h = 51, $fn = 3, center=true);
        intersection() {
            rotate([0, 90, 0]) cylinder(r=2.5, h=60, center = true, $fn=20);
            cube([50, 5, 4.2], center=true);
        }
        intersection() {
            rotate([90, 0, 0]) scale([1, 0.4, 1])
                cylinder(r=2.5, h=60, center = true, $fn=20);
            cube([5, 59, 1.2], center=true);
        }
    }
}

module skew() {
    rotate([slope, 0, 0])
        multmatrix([[1, 0, 0], [0, 1, 0], [0, 1/sqrt(3), 1]])
        cube([2, diamond_offset * 2, 45], center=true);
}

module strut() {
    thick = 3;
    translate([0, 0, thick/2]) cube([diamond_offset * 2, thick, thick], center=true);
}

module runner() {
    rotate([90, 0, 90]) cylinder(r=10, h=6, center=true, $fn=3);
}

