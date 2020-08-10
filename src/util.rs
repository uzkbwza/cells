use sdl2::rect::Point;

pub fn distance(x0: i32, y0: i32, x1: i32, y1: i32) -> f32 {
    return f32::sqrt(
            ((x1 - x0).pow(2) as f32)
            +
            ((y1 - y0).pow(2) as f32)
        )
}

pub fn line(x0: i32, y0: i32, x1: i32, y1: i32) -> Vec<Point> {
    if y0 == y1 && x0 == x1 {
        vec![Point::new(x0, y0)]
    } else if y0 == y1 {
        h_line(x0, x1, y0)
    } else if x0 == x1 {
        v_line (x0, y0, y1)
    } else {
        bresenham(x0, y0, x1, y1)
    }

}

fn h_line(mut x0: i32, mut x1: i32, y: i32) -> Vec<Point> {
    let mut points = Vec::new();
    if x0 > x1 {
        std::mem::swap(&mut x0, &mut x1);
    }
    for x in x0..x1 {
        points.push(Point::new(x, y));
    }
    points
}

fn v_line(x: i32, mut y0: i32, mut y1: i32) -> Vec<Point> {
    let mut points = Vec::new();
    if y0 > y1 {
        std::mem::swap(&mut y0, &mut y1);
    }
    for y in y0..y1 {
        points.push(Point::new(x, y));
    }
    points
}

fn bresenham(mut x0: i32, mut y0: i32, x1: i32, y1: i32) -> Vec<Point> {
    // Bresenham's line drawing algorithm
    let mut points = Vec::new();
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs(); 
    let sx = if x0 < x1 { 1 } else { -1};
    let sy = if y0 < y1 { 1 } else { -1};
    let mut err = dx + dy;
    let mut e2: i32;
    loop {
        points.push(Point::new(x0, y0));
        if x0==x1 && y0==y1 { break }
        e2 = 2*err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
    points
}
