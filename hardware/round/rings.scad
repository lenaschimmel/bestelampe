$fn = 120;
sc=0.0847;

// oben maximaler innendurchmesser: 108mm
// auf Ringhöhe maximaler innendurchmesser: 126mm		
// mitte maximaler innendurchmesser: 140mm
r_inner_mid = 70;
// unten maximaler innendurchmesser: 156mm, außen 163mm
r_inner_bottom = 78;
r_outer_bottom = 81.5;

// new approach
// upper r 66mm
// lower r 75.5mm

module ring(h, r_outer, r_inner) {
	difference() {
		cylinder(h=h, r=r_outer);
		translate([0,0,-1]) cylinder(h=h+2, r=r_inner);
	}
}

module ring_with_slit(r_outer, r_inner, thickness, h_floor, h_slit) {
	difference() {
		ring(h=h_floor + h_slit, r_outer + thickness, r_inner - thickness);
		translate([0,0,h_floor]) ring(h=h_floor+h_slit, r_outer, r_inner);
	}
}

//ring_with_slit(66.0, 65.5, 0.8, 1, 3);
//ring_with_slit(75.0, 74.5, 0.8, 1, 3);

//ring_with_slit(r_outer_bottom, r_inner_bottom,2);
intersection() {
	union() {
		// for the first diffusor
		ring_with_slit(r_inner_mid - 1, r_inner_mid-1.8, 0.8, 1, 4);

		// inside the diffusor:
		ring(3, r_inner_mid - 2, r_inner_mid - 4);

		//translate([0, -10, 0]) ring_with_slit(r_inner_mid - 1, r_inner_mid-1.8, 0.8, 1, 4); 

		// for the transparent shell:
		ring_with_slit(r_outer_bottom + 0.5, r_inner_bottom - 0.5, 1.6, 1, 3);

		// for the inner reflector:
		ring_with_slit(27, 25, 1.6, 1, 2);


		// outer spokes:
		for (i=[0:20:359]) {
			rotate(i, [0,0,1]) translate([-2, r_inner_mid-2, 0]) cube([4,10,1]);
		}

		// inner spokes:
		for (i=[0:60:359]) {
			#rotate(i, [0,0,1]) translate([-2, 28, 0]) cube([4,40,3]);
		}
		
	}
	//translate([-30, 0, 0]) cube([60,400,20]);
}
