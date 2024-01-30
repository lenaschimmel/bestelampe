sc=0.0847;

//union() {
//	intersection() {
//		translate([0,0,30]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="Hausnummernreflektor"); 
//		translate([0,0,30]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="Hauptreflektor"); 
//		translate([0,0,30]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="Abstandhalter"); 
//		translate([-4, -400, -10]) cube([8,800,66]);
//	}
//	#translate([-2, -55, 14.5]) cube([4,110,2.5]);
//}
//color("blue") translate([-140,45,47]) linear_extrude(height=5, convexity = 10, $fn = 80) scale(sc) import("schnitt.svg", convexity=3, id="Hausnummernkreis");
//difference() { 
//	translate([0,0,130]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt2.svg", convexity=3, id="ObererRing2"); 	
//	translate([0,0,130]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="Innendiffusor"); 	
//	translate([0,0,30]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="HausnummerndeckelAusschnitt"); 
//	translate([0,0,30]) rotate_extrude(convexity = 10, $fn = 180) scale(sc) import("schnitt.svg", convexity=3, id="Hausnummernscheibe"); 
// 	translate([-140,45,0]) linear_extrude(height=3.14, convexity = 10, $fn = 80) scale(sc) import("schnitt.svg", convexity=3, id="Hausnummer");
//}


//translate([-140,45,0]) linear_extrude(height=93, convexity = 10, $fn = 80) scale(sc) import("schnitt.svg", convexity=3, id="Inlay");
//translate([-140,45,0]) linear_extrude(height=2, convexity = 10, $fn = 80) scale(sc) import("schnitt.svg", convexity=3, id="InlayRand");
//translate([-140,45,0]) linear_extrude(height=2.415, convexity = 10, $fn = 80) scale(sc) import("schnitt.svg", convexity=3, id="Nubsies");


 translate([-140,45,0]) linear_extrude(height=2, convexity = 10, $fn = 20) scale(sc) import("zweiterschnitt.svg", convexity=3, id="Durchgehend");
 translate([-140,45,0]) linear_extrude(height=6, convexity = 10, $fn = 20) scale(sc) import("zweiterschnitt.svg", convexity=3, id="Aufbau");

