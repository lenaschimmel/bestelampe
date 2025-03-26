// Test #1, with LED strip "warm white, cold white"
use <./paraboloid.scad>

color("blue") translate([-2.5,-2.5,-1.2]) cube([5,5,1.2]);

w = 16.75;
l = 16.75;

for (i=[0:6]) {
	translate([i*l,0,0])
	render() difference() {
		translate([-l/2,-w/2,0]) cube([l,w,15]);
		paraboloid(25, 0.65, 1, 1.3, 80);
	}
}

