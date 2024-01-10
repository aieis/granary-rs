pub mod mat;

use mat::mat::SparseMatrix;
use num::clamp;
use rand::{Rng, distributions::Bernoulli, seq::SliceRandom};
use std::io::{self, Write, Stdout};
use crossterm::{
    execute, queue,
    terminal, cursor, style, event::Event
};

struct Granary {
    cells : Vec<CellContainer>,
    shots : Vec<Point>,
    map : SparseMatrix<u32>,
    width: f64,
    height: f64,
}

struct CellContainer
{
    id : u32,
    cell : Cell,
    loc : Point,
}

struct Point {
    x: f64,
    y: f64
}

struct Cell {
    nin: usize,
    nout: usize,
    nknots: usize,
    knots: Vec<i32>,
}

struct Canvas {
    height : usize,
    width : usize,
    stride : usize,
    data: Vec<char>,
    stdout : Stdout,
}

fn make_cell(nin : usize, nout : usize) -> Cell
{
    let nknots : usize =  (rand::thread_rng().gen_range(10..=100) + nin + nout) as usize;
    let d = Bernoulli::new(0.5).unwrap();
    let mut knots : Vec<i32> = rand::thread_rng().sample_iter(d).take(nknots*nknots).map(|b| if b {1} else {0}).collect();
    
    for i in 0..nknots {
        let idx = i*nknots + i;
        knots[idx] = 0;
    }

    return Cell {
        nin,
        nout,
        nknots,
        knots,
    }
}

fn shock(cell : &mut Cell, input : Vec<i32>)
{
    if input.len() !=  cell.nin {
        panic!();
    }

    for i in 0..cell.nin {
        let idx = i * cell.nknots + i;
        cell.knots[idx] = input[i];
    }

    for i in 0..cell.nknots {
        let idx = i * cell.nknots + i;
        let val : i32 = cell.knots[i*cell.nknots..(i+1)*cell.nknots].iter().sum();
        cell.knots[idx] = val - cell.knots[idx];
    }
}

fn pulse(cell : &mut Cell) {
    for i in 0..cell.nknots {
        let idx = i * cell.nknots + i;
        cell.knots[idx] = cell.knots[idx] - 1;
    }
}

fn query(cell : &Cell) -> Vec<i32> {
    let mut output : Vec<i32> = Vec::new();
    for i in 0..cell.nout {
        let j = cell.nknots - cell.nout + i;
        let idx = j * cell.nknots + j;
        output.push(cell.knots[idx]);
    }

    return output;
}

// fn print_cell(cell : &Cell) {
//     for i in 0..cell.nknots {
//         println!("{:?}", &cell.knots[i*cell.nknots..(i+1)*cell.nknots]);
//     }
// }

// fn random_point(width : f64, height : f64) -> Point {
//     return Point {
//         x : rand::thread_rng().gen_range(0..width as i64 - 1) as f64,
//         y : rand::thread_rng().gen_range(0..height as i64 - 1) as f64
//     }
// }

fn random(mx : i32) -> f64 {
    return rand::thread_rng().gen_range(0..mx) as f64;
}

// fn clamp(val : f64, mn : f64, mx : f64) -> f64 {
//     return 
// }

fn nourish() {
    let width : f64 = 4000.0;
    let height : f64 = 4000.0;

    let mut posx : Vec<u32> = (0..width as u32).collect();
    let mut posy : Vec<u32> = (0..height as u32).collect();

    posx.shuffle(&mut rand::thread_rng());
    posy.shuffle(&mut rand::thread_rng());

    let num_cells = 1000;
    let cells : Vec<CellContainer> = (2..num_cells as usize + 2)
        .map(|id|
             CellContainer {
                 id: id as u32,
                 cell : make_cell(4, 4),
                 loc : Point {x: posx[id] as f64, y: posy[id] as f64}
             })
        .collect();

    let shots : Vec<Point> = (0..1000 as usize)
        .map(|ind| {Point {x: posx[ind + num_cells] as f64, y: posy[ind + num_cells] as f64}})
        .collect();

    let beat = 0;
    let shock_rate = 20;

    let mut gran = Granary {
        cells,
        shots,
        width,
        height,
        map : SparseMatrix::new(width as usize, height as usize, 0)
    };

    for cellc in gran.cells.iter() {
        let x = cellc.loc.x as usize;
        let y = cellc.loc.y as usize;
        gran.map.insert(x, y, cellc.id);
    }

    for loc in gran.shots.iter() {
        let x = loc.x as usize;
        let y = loc.y as usize;
        gran.map.insert(x, y, 1);
    }


    let cw = 120;
    let ch = 60;

    let mut canvas = make_canvas(cw, ch);

    loop {
        
        if beat % shock_rate == 0 {
            for cellc in gran.cells.iter_mut() {
                let p : &Point = &(gran.shots[random(gran.shots.len() as i32) as usize]);
                let inpb : Vec<bool> =  vec![p.x < cellc.loc.x, p.y < cellc.loc.y, p.x > cellc.loc.x, p.y > cellc.loc.y];
                let inp : Vec<i32> = inpb.iter().map(|b| if *b {1} else {0}).collect();
                shock(&mut cellc.cell, inp);
            }
        } else {
            for cellc in gran.cells.iter_mut() {
                pulse(&mut cellc.cell);
            }
        }

        for cellc in gran.cells.iter_mut() {
            let res = query(&cellc.cell);

            let tx = clamp(cellc.loc.x + (res[0] - res[2]) as f64, 0.0, width - 1.0);
            let ty = clamp(cellc.loc.y + (res[1] - res[3]) as f64, 0.0, height - 1.0);

            if gran.map[[tx as usize, ty as usize]] != 0 || (tx == cellc.loc.x && ty == cellc.loc.y) {
                continue;
            }

            gran.map.insert(cellc.loc.x as usize, cellc.loc.y as usize, 0);
            
            cellc.loc.x = tx;
            cellc.loc.y = ty;
            gran.map.insert(tx as usize, ty as usize, cellc.id);
        }
        
        if !draw(&mut canvas, &gran) {
            break;
        }

    }
    
    cleanup_canvas(&mut canvas);
}

fn make_canvas(width : usize, height : usize) -> Canvas {
    let stride = width + 3;
    let size = (width + 3) * (height + 2);
    let mut data = Vec::with_capacity(size);
    data.resize(size, ' ');

    for i in 0..width+3 {
        let idx_top = i;
        let idx_bot = (height + 1) * stride + i;
        data[idx_top] = '=';
        data[idx_bot] = '=';
    }

    for i in 0..height+2 {
        let idx_left = i * stride;
        let idx_right = (i + 1) * stride - 2;
        data[idx_left] = '|';
        data[idx_right] = '|';
        data[idx_right + 1] = '\n';
    }

        
    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
    
    return Canvas {width, height, stride, data, stdout};
}

fn cleanup_canvas(canvas : &mut Canvas) {
    canvas.stdout.flush().unwrap();
}

fn draw(canvas : &mut Canvas, gran : &Granary) -> bool
{
    let mut board = canvas.data.clone();

    let cw = canvas.width;
    let ch = canvas.height;
    let stride = canvas.stride;
    
    for shot in gran.shots.iter() {
        let posx = (shot.x / gran.width * cw as f64) as usize;
        let posy = (shot.y / gran.height * ch as f64) as usize;
        let idx = (posy + 1) * stride + (posx + 1);
        board[idx] = '+';
    }

    for cellc in gran.cells.iter() {
        let posx = (cellc.loc.x / gran.width * cw as f64) as usize;
        let posy = (cellc.loc.y / gran.height * ch as f64) as usize;
        let idx = (posy + 1) * stride + (posx + 1);
        board[idx] = 'O';
    }

    let boardstr = String::from_iter(board);

    queue!(canvas.stdout, cursor::MoveTo(0,0), style::Print(boardstr)).unwrap();
    if crossterm::event::poll(std::time::Duration::from_millis(1)).unwrap() {
        return match crossterm::event::read().unwrap() {
            Event::Key(_) => false,
            _ => true
        }
    }

    return true;
}

fn main() {
    nourish();
    println!("Done");
    
}
