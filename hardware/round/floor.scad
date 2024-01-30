$fn=180;
sc=0.0847;
//sc=0.24;

// Sprayring:
//translate([0,0,147]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="Spraydeckel");

module ring(h, r_outer, r_inner) {
	difference() {
		cylinder(h=h, r=r_outer);
		translate([0,0,-1]) cylinder(h=h+2, r=r_inner);
	}
}

intersection() {
	difference() {
		union() {
			translate([0,0,147]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="Boden");
			translate([0,0,-4]) ring(5, 85, 32.5);
		}
 		for (i=[0:90:359]) {
			rotate(i, [0,0,1]) translate([0, 29.5, 7.5]) rotate(-110, [1,0,0]) cylinder(h=12, d=6.5,$fn=12);
			rotate(i, [0,0,1]) translate([0, 40, 3.5]) rotate(-100, [1,0,0]) cylinder(h=6, d=6.5,$fn=12);
			rotate(i, [0,0,1]) translate([0, 45, 2.5]) rotate(-92.5, [1,0,0]) cylinder(h=16, d1=6.5, d2=8.5,$fn=12);
			rotate(i, [0,0,1]) scale([1,1,1.5]) translate([0, 60, 2.75]) rotate(-80, [1,0,0]) cylinder(h=14, d=8.5,$fn=12);
			//rotate(i, [0,0,1]) translate([0, 65, -1.5]) rotate(0, [1,0,0]) cylinder(h=12, d=10.5,$fn=12);
			
		}
		
		//i=0; {
			// for (n=[-1:1]) {
 			// 	rotate(i, [0,0,1]) translate([n*4, 31.5, 7.5]) rotate(-96.5, [1,0,0]) cylinder(h=42.1, d=2.25,$fn=12);
			// 	rotate(i, [0,0,1]) translate([n*4, 32.5, 7.5]) rotate(45, [1,0,0]) cylinder(h=10, d=2.25,$fn=12);
			// 	translate([-9, 72.5, 1.25]) rotate(10, [1,0,0]) cube([18,4,20]);
			// }
 		//}
		//rotate(-45, [0,0,1]) translate([0, 78, -1]) rotate(45, [1,0,0]) cylinder(h=7, d=2.5,$fn=8);
		//rotate(-45, [0,0,1]) translate([0, 93, -0.5]) rotate(85, [1,0,0]) cylinder(h=17, d=2.5,$fn=8);
	} 
	//translate([-5, -400, 0]) cube([10,800,16]);
	//translate([0,0,-10]) cube([800,800,100]);	 
}
//#cylinder(h=2,r=85,$fn=60);

