/*
    parabola module from https://mastering-openscad.eu/buch/example_09/
    
    - focus_point,    focus point as 3D-vector
    - base_area,      dimension of the base are as 2D-vector
    - resolution,     number of grid points as 2D-vector

    FIXME This code generates faces with wrong orientations, which
    cannot be used in OpenSCAD for boolean operations, etc.
*/
module parabola( 
    focus_point, 
    base_area, 
    resolution = [10, 10], 
    thickness  = 0   
){
    
    function parabola_point( focus_point, base_point ) =
        let ( dist_fb = norm(focus_point - base_point) )
        [
            base_point.x,
            base_point.y,
            ( dist_fb * dist_fb ) / ( 2 * (focus_point.z - base_point.z) )
        ];
    
    function flip(vec) = [ vec[3], vec[2], vec[1], vec[0] ];

    parabola_points = [
        for ( 
            y = [0 : base_area.y / resolution.y : base_area.y + 0.1], 
            x = [0 : base_area.x / resolution.x : base_area.x + 0.1] 
        )
        parabola_point( focus_point, [x,y,0] )
    ];

    base_points = [
        for ( 
            y = [0 : base_area.y / resolution.y : base_area.y + 0.1], 
            x = [0 : base_area.x / resolution.x : base_area.x + 0.1] 
        ) 
        let ( p = parabola_point( focus_point, [x,y,0] ) )
        if (thickness > 0)
            [x, y, p.z - thickness]
        else
            [x, y, 0]
    ];

    size_x = resolution.x + 1;
                
    parabola_faces = [
        for ( 
            y = [0 : resolution.y - 1], 
            x = [0 : resolution.x - 1] 
        )
        [ 
            y    * size_x + x, 
            y    * size_x + x + 1,
           (y+1) * size_x + x + 1,
           (y+1) * size_x + x
        ]
    ];

    size_ppoints = len( parabola_points );
    
    base_faces = [
        for ( 
            y = [0 : resolution.y - 1], 
            x = [0 : resolution.x - 1] 
        )
        [ y    * size_x + x     + size_ppoints, 
          y    * size_x + x + 1 + size_ppoints,
         (y+1) * size_x + x + 1 + size_ppoints,
         (y+1) * size_x + x     + size_ppoints]
    ];


    side_faces_1 = [
        for ( x = [0 : resolution.x - 1] )
        [ x, 
          x + 1, 
          x + 1 + size_ppoints, 
          x     + size_ppoints ]
    ];


    last_row = resolution.y * size_x;
        
    side_faces_2 = [
        for ( x = [0 : resolution.x - 1] )
        [ 
            last_row + x, 
            last_row + x + 1, 
            last_row + x + 1 + size_ppoints, 
            last_row + x     + size_ppoints
        ]
    ];

    side_faces_3 = [
        for ( y = [0 : resolution.y - 1] )
        [ 
            y      * size_x, 
           (y + 1) * size_x, 
           (y + 1) * size_x + size_ppoints, 
            y      * size_x + size_ppoints 
        ]
    ];

    last_col = resolution.x;

    side_faces_4 = [
        for ( y = [0 : resolution.y - 1] )
        [ 
            last_col +  y      * size_x, 
            last_col + (y + 1) * size_x, 
            last_col + (y + 1) * size_x + size_ppoints, 
            last_col +  y      * size_x + size_ppoints
        ]
    ];

    polyhedron(
        points = concat( parabola_points, base_points ), 
        faces  = concat( parabola_faces, 
                         base_faces, 
                         side_faces_1,
                         side_faces_2,
                         side_faces_3,
                         side_faces_4)
    );
    
}

render()
parabola( 
    focus_point = [75,75,100],
    base_area   = [150,150],
    thickness   = 0
);


