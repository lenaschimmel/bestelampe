// Test #1, with LED strip "warm white, cold white"
use <./paraboloid.scad>

//color("blue") translate([-1,-1.75,-0.8]) cube([2,3.5,0.8]);
for (i=[0:5]) {
	translate([i*10.4,0,0])
	render() difference() {
		translate([-10.4/2,-15/2,0]) cube([10.4,15,12]);
		paraboloid(20, 0.65, 1, 0.6, 80);
	}
}

