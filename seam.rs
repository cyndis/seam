struct Matrix<T> {
  mut data: ~[T],
  w: uint,
  h: uint
}

impl<T: Copy> Matrix<T> {
  pure fn at(x: uint, y: uint) -> T {
    self.data[y*self.w+x]
  }
  fn set(x: uint, y: uint, c: T) {
    self.data[y*self.w+x] = c;
  }
  pure fn width() -> uint { self.w }
  pure fn height() -> uint { self.h }
}

struct Color {
  r: float, g: float, b: float
}

impl Color {
  fn map(f: fn&(t: float) -> float) -> Color {
    Color { r: f(self.r), g: f(self.g), b: f(self.b) }
  }

  pure fn brightness() -> float {
    0.0722 * self.r + 0.7152 * self.g + 0.2126 * self.b
  }
}

type Image = Matrix<Color>;

fn Image(width: uint, height: uint) -> Image {
  let mut i = Matrix { data: ~[], w: width, h: height };
  let black = Color{r: 0.0, g: 0.0, b: 0.0};
  i.data.grow(width * height, &black);
  move i
}

enum Direction {
  Left,
  Right,
  Up
}

impl Image {
  fn energy() -> Matrix<float> {
    let m = Matrix{ data: ~[], w: self.w, h: self.h };
    m.data.grow(self.w * self.h, &0.0);

    for uint::range(0, self.h) |y| {
      for uint::range(0, self.w) |x| {
        let c = self.at(x,y);
        let l = (if x > 0 { self.at(x-1,y) } else { c }).brightness();
        let r = (if x < self.w-1 { self.at(x+1,y) } else { c }).brightness();
        let u = (if y > 0 { self.at(x,y-1) } else { c }).brightness();;
        let d = (if y < self.h-1 { self.at(x,y+1) } else { c }).brightness();

        let x_energy = -0.5 * l + 0.5 * r;
        let y_energy = -0.5 * d + 0.5 * u;
        let energy = float::sqrt(x_energy*x_energy + y_energy*y_energy) /
                     float::sqrt(0.5);
        m.set(x,y,energy);
      }
    }

    move m
  }

  fn best_seam() -> (float, ~[uint]) {
    let ene = self.energy();
    let map: Matrix<(float, Option<Direction>)> =
      Matrix{ data: ~[], w: self.w, h: self.h };
    map.data.grow(self.w * self.h, &(0.0, None));

    for uint::range(0, self.h) |y| {
      for uint::range(0, self.w) |x| {
        if y == 0 {
          map.set(x, y, (ene.at(x,y), None));
        } else {
          let l = if x > 0 { ene.at(x-1,y-1) } else { float::infinity };
          let t = ene.at(x,y-1);
          let r = if x < self.w-1 { ene.at(x+1,y-1) } else { float::infinity };
          let min = [l,t,r].min();
          let my = ene.at(x, y);
          if min == l {
            map.set(x, y, (my + l, Some(Left)));
          } else if min == r {
            map.set(x, y, (my + r, Some(Right)));
          } else {
            map.set(x, y, (my + t, Some(Up)));
          }
        }
      }
    }

    let mut best_seam = ~[];
    let mut best_energy = float::infinity;

    for uint::range(0, self.w) |x| {
      let mut total_energy = 0.0;
      let mut pixels = ~[];
      let mut cx = x, cy = self.h-1;
      loop {
        let data = map.at(cx,cy);
        total_energy += data.first();
        if total_energy > best_energy { break; }

        pixels.push(cx);

        match data.second() {
          None => break,
          Some(Left) => cx -= 1,
          Some(Right) => cx += 1,
          Some(Up) => ()
        }

        cy -= 1;
      }

      if total_energy < best_energy {
        best_energy = total_energy;
        best_seam = move pixels;
      }
    }

    (best_energy, vec::reversed(best_seam))
  }
}

fn carve(image: &Image) -> Image {
  let (_, seam) = image.best_seam();
  let i = Image(image.width() - 1, image.height());
  for uint::range(0, image.height()) |y| {
    for uint::range(0, image.width()) |x| {
      if x < seam[y] {
        i.set(x, y, image.at(x, y));
      } else if x > seam[y] {
        i.set(x-1, y, image.at(x, y));
      }
    }
  }

  move i
}

fn carven(image: &Image, n: uint) -> Image {
  let mut im: Image = copy *image;
  for n.times || {
    let c = carve(&im);
    im = move c;
  }
  move im
}

pure fn round(f: float) -> int {
  let rem = f - (f as int as float);
  if rem >= 0.5 { f as int + 1}
  else { f as int }
}

use io::WriterUtil;
impl Image {
  fn save_ppm(path: &path::Path) {
    let wr = io::file_writer(path, [io::Create]).get();
    wr.write_line("P3");
    wr.write_line(fmt!("%u %u", self.w, self.h));
    wr.write_line("255");
    for uint::range(0, self.h) |y| {
      for uint::range(0, self.w) |x| {
        let color = self.at(x,y).map(|c| c * 255.0);
        let c = [round(color.r), round(color.g), round(color.b)];
        wr.write_line(fmt!("%d %d %d", c[0], c[1], c[2]));
      }
    }
  }
}

use io::ReaderUtil;
fn load_ppm(path: &path::Path) -> Result<Image,~str> {
  let rd = io::file_reader(path).get();

  if "P3" != rd.read_line() {
    Err(~"image is not of type P3")
  } else {
    let dims = str::split_char(rd.read_line(), ' ');
    let image = Image(uint::from_str(dims[0]).get(),
                       uint::from_str(dims[1]).get());
    let max = uint::from_str(rd.read_line()).get() as float;
    let mut cx = 0, cy = 0;
    while !rd.eof() {
      let cs = str::split_char(rd.read_line(), ' ');
      for uint::range(0, cs.len() / 3) |i| {
        let ucolor = [uint::from_str(cs[3*i]).get(),
                      uint::from_str(cs[3*i+1]).get(),
                      uint::from_str(cs[3*i+2]).get()];
        let color = Color{ r: (ucolor[0] as float) / max,
                           g: (ucolor[1] as float) / max,
                           b: (ucolor[2] as float) / max };
        image.set(cx, cy, color);
        cx += 1;
        if cx == image.width() {
          cx = 0;
          cy += 1;
          if cy == image.height() {
            return Ok(copy image); // :(
          }
        }
      }
    }

    Err(~"image doesn't contain enough pixel data")
  }
}

fn main() {
  let args = os::args();
  if args.len() < 4 {
    io::println("usage: seam <input.ppm> <cols> <output.ppm>");
    io::println("where rows is the number of columns to carve out");
  } else {
    let im = load_ppm(&path::Path(args[1]));
    let carved = carven(im.get_ref(), uint::from_str(args[2]).get());
    carved.save_ppm(&path::Path(args[3]));
  }
}
