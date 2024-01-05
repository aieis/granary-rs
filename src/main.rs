use num::clamp;
use rand::{distributions::Bernoulli, Rng};
use std::io::{self, Write, Stdout};
use crossterm::{
    execute, queue,
    terminal, cursor, style::{self, Stylize}, event::Event
};


struct Granary {
    cells : Vec<CellContainer>,
    shots : Vec<Point>,
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

fn print_cell(cell : &Cell) {
    for i in 0..cell.nknots {
        println!("{:?}", &cell.knots[i*cell.nknots..(i+1)*cell.nknots]);
    }
}

fn random_point(width : f64, height : f64) -> Point {
    return Point {
        x : rand::thread_rng().gen_range(0..width as i64) as f64,
        y : rand::thread_rng().gen_range(0..height as i64) as f64
    }
}

fn random(mx : i32) -> f64 {
    return rand::thread_rng().gen_range(0..mx) as f64;
}

// fn clamp(val : f64, mn : f64, mx : f64) -> f64 {
//     return 
// }

fn nourish() {
    let width : f64 = 2000.0;
    let height : f64 = 2000.0;
    let cells : Vec<CellContainer> = (0..10)
        .map(|id|
             CellContainer {
                 id,
                 cell : make_cell(4, 4),
                 loc : random_point(width, height)})
        .collect();

    let shots : Vec<Point> = (0..10)
        .map(|_| {random_point(width, height)})
        .collect();

    let beat = 0;
    let shock_rate = 20;

    let mut gran = Granary {
            cells,
            shots,
            width,
            height
    };


    let mut stdout = io::stdout();
    execute!(stdout, terminal::Clear(terminal::ClearType::All)).unwrap();
    
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
            cellc.loc.x = clamp(cellc.loc.x + (res[0] - res[2]) as f64, 0.0, width);
            cellc.loc.y = clamp(cellc.loc.y + (res[1] - res[3]) as f64, 0.0, height);            
        }
        
        if !draw(&gran, &mut stdout) {
            break;
        }
    }

    stdout.flush().unwrap();
}


fn draw(gran : &Granary, stdout : &mut Stdout) -> bool
{
    const CW : usize = 120;
    const CH : usize = 120;

    let mut board = [[' '; CW+1]; CH+1];

    for cellc in gran.cells.iter() {
        let posx = (cellc.loc.x / gran.width * CW as f64) as usize;
        let posy = (cellc.loc.y / gran.height * CH as f64) as usize;
        board[posy][posx] = 'O';
    }

    for shot in gran.shots.iter() {
        let posx = (shot.x / gran.width * CW as f64) as usize;
        let posy = (shot.y / gran.height * CH as f64) as usize;
        board[posy][posx] = '+';
    }

    let mut boardlist : Vec<String> = vec![String::from_iter(['='; CW+1])];
    let mut board_s : Vec<String> = board.iter().map(|x| String::from_iter(x)).collect();
    boardlist.append(&mut board_s);
    boardlist.push(String::from_iter(['='; CW+1]));
    
    let boardstr = boardlist.join("\n");

    queue!(stdout, cursor::MoveTo(0,0), style::Print(boardstr)).unwrap();
    if crossterm::event::poll(std::time::Duration::from_millis(1)).unwrap() {
        return match crossterm::event::read().unwrap() {
            Event::Key(_) => false,
            _ => true
        }
    }

    return true;
    
    // println!("{}", String::from_iter(['='; CW+1]));
    // for row in board {
    //     println!("{}", String::from_iter(row));
    // }
    // println!("{}", String::from_iter(['='; CW+1]));
    
}

fn main() {
    nourish();
    println!("Done");
    
}
