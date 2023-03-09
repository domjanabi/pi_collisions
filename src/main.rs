use olc::Vd2d;
use olc_pge as olc;
use web_audio_api::context::{BaseAudioContext, AudioContext};
use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

const fps:f32 = 60.0;

fn generate_ticks(data: &mut[f32], frequency: usize, length: usize)
{   
    for sample in data.iter_mut()
    {
        *sample = 0.0;
    }
    let len = length.min(data.len()/(frequency*2).max(1));
    for f in 0..frequency
    {
        for l in 0..len
        {
            let offset = f * (data.len()/frequency);
            data[l.min(data.len()-offset-1)+offset] = 0.3 + 0.7/frequency as f32;
        }
    }
}

fn get_exponent_input() -> usize
{
    let upper_bound = 7;
    println!("Enter an exponent for 100^x between 0 and {upper_bound}");
    let mut input = String::new();
    
    if let Some(s) = std::env::args().take(2).collect::<Vec<String>>().get(1)
    {
        input = s.clone();
    }
    else
    {
        std::io::stdin().read_line(&mut input).unwrap();
    }
    loop
    {
        match input.trim().parse::<usize>()
        {
            Err(_e) => println!("Invalid input. Input must be a natural number between 0 and {upper_bound}."),
            Ok(num) => if num <= 7 {break num} else{println!("Value was outside of permissible range. Enter a value between 0 and {upper_bound}.");},
        };
        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();
    }
}

fn main()
{
    // set up AudioContext with optimized settings for your hardware
    let context = AudioContext::default();
    let exponent = get_exponent_input();


    let window = Window
    {
        m1: 1.0,
        m2: 100.0f64.powi(exponent as i32),
        pos1: Vd2d::new(15.0, 370.0),
        pos2: Vd2d::new(55.0, 350.0),
        size1: Vd2d::new(30.0,30.0),
        size2: Vd2d::new(50.0, 50.0),
        vel1: Vd2d::new(0.0, 0.0),
        vel2: Vd2d::new(-1.0, 0.0),
        counter: 0,
        context,
        frame_counter: 0,
        per_second_counter: 0,
        collisions_per_second: 0.0,
        avg_frame_time: 0.0,
    };
    olc::PixelGameEngine::construct(window, 400, 400, 2 ,2).start();
}

struct Window
{
    m1: f64,
    m2: f64,
    pos1: Vd2d,
    pos2: Vd2d,
    size1: Vd2d,
    size2: Vd2d,
    vel1: Vd2d,
    vel2: Vd2d,
    counter: usize,
    context: AudioContext,
    frame_counter: usize,
    per_second_counter: usize,
    collisions_per_second: f32,
    avg_frame_time: f32,
    // buffer: AudioBuffer,
    // buffer_source: AudioBufferSourceNode,
}

fn castvec_i32(v: Vd2d) -> olc::Vi2d
{
    return olc::Vi2d::new(v.x as i32, v.y as i32);
}

impl Window
{
    fn physics_step(&mut self, pge: &mut olc::PixelGameEngine) -> usize
    {
        let width = pge.screen_width() as f64;
        let height = pge.screen_height() as f64;
        let mut collision_counter = 0;
        let iterations = 100.max(self.m2.sqrt() as i32 / 10).min(100000);
        for i in 0..iterations
        {
            let d = 1.0/iterations as f64;
            //bounce
            if self.pos1.x < 0.0
            {
                self.vel1.x = self.vel1.x.abs();
                self.pos1.x = 0.0;
                collision_counter += 1;
            }
            if self.pos1.y < 0.0
            {
                self.vel1.y = self.vel1.y.abs();
                self.pos1.y = 0.0;
                collision_counter += 1;
            }
            if self.pos1.x + self.size1.x > width
            {
                self.vel1.x = -self.vel1.x.abs();
                self.pos1.x = width - self.size1.x;
                collision_counter += 1;
            }
            if self.pos1.y + self.size1.y > height
            {
                self.vel1.y = -self.vel1.y.abs();
                self.pos1.y = height - self.size1.y;
                collision_counter += 1;
            }

            // if self.pos2.x < 0.0
            // {
            //     self.vel2.x = self.vel2.x.abs();
            //     self.pos2.x = 0.0;
            //     collision_counter += 1;
            // }
            // if self.pos2.y < 0.0
            // {
            //     self.vel2.y = self.vel2.y.abs();
            //     self.pos2.y = 0.0;
            //     collision_counter += 1;
            // }
            // if self.pos2.x + self.size2.x > width
            // {
            //     self.vel2.x = -self.vel2.x.abs();
            //     self.pos2.x = width - self.size2.x;
            //     //collision_counter += 1;
            //     //TODO: in order to actually calculate pi with this, this += 1 right above shouldn't count
            // }
            // if self.pos2.y + self.size2.y > height
            // {
            //     self.vel2.y = -self.vel2.y.abs();
            //     self.pos2.y = height - self.size2.y;
            //     //collision_counter += 1;
            //     //TODO: in order to actually calculate pi with this, this += 1 right above shouldn't count
            // }

            if self.pos1.x < self.pos2.x + self.size2.x
            && self.pos1.x + self.size1.x > self.pos2.x
            && self.pos1.y < self.pos2.y + self.size2.y
            && self.pos1.y + self.size1.y > self.pos2.y
            {
                // v1 = (m1-m2)/(m1+m2)*u1 + (2*m2)/(m1+m2)*u2
                // v2 = (m2-m1)/(m1+m2)*u2 + (2*m1)/(m1+m2)*u1
                let mass_sum = self.m1 + self.m2;
                let v1 = self.vel1 * ((self.m1 - self.m2)/mass_sum) + self.vel2 * ((2.0*self.m2)/mass_sum);
                let v2 = self.vel2 * ((self.m2 - self.m1)/mass_sum) + self.vel1 * ((2.0*self.m1)/mass_sum);
                self.vel1 = v1;
                self.vel2 = v2;
                collision_counter += 1;
            }

            self.pos1 += self.vel1 * d;
            self.pos2 += self.vel2 * d;
            
            if iterations % (iterations / 100) == 0
            {
                let pos = castvec_i32(self.pos1);
                let size = castvec_i32(self.size1);
                let r = (i*155/iterations)as u8 + 100;
                let g = ((i-5).max(0)*200/(iterations-5))as u8 + 55;
                let b = (i*255/iterations) as u8;
                pge.fill_rect_v(pos, size, olc::Pixel::rgb(r,g,b));   
            }
        }
        
        return collision_counter;
    }
}


impl olc::PGEApplication for Window
{
    const APP_NAME: &'static str = "PI Collisions";
    fn on_user_update(&mut self, pge: &mut olc::PixelGameEngine, elapsed_time: f32) -> bool
    {
        self.avg_frame_time = (self.avg_frame_time * (fps-1.0) + elapsed_time) / fps;
        std::thread::sleep(std::time::Duration::from_secs_f32((1.0/fps - self.avg_frame_time).max(0.0)));
        pge.clear(olc::BLACK);
        let collision_amount = self.physics_step(pge);

        if self.frame_counter % fps as usize == 0
        {
            self.frame_counter = 0;
            self.collisions_per_second = self.per_second_counter as f32;
            self.per_second_counter = 0;
        }

        self.frame_counter += 1;
        self.per_second_counter += collision_amount;

        let mut buffer = self.context.create_buffer(1, (self.context.sample_rate() / fps) as usize, self.context.sample_rate());

        generate_ticks(buffer.get_channel_data_mut(0),collision_amount,10);

        // play the buffer at given volume
        let volume = self.context.create_gain();
        volume.connect(&self.context.destination());
        volume.gain().set_value(0.5);

        let buffer_source = self.context.create_buffer_source();
        buffer_source.connect(&volume);
        buffer_source.set_buffer(buffer);

        // start the sources
        buffer_source.start();

        self.counter += collision_amount;
        pge.draw_string(pge.screen_width() as i32 - 100, 20, &format!("{}", self.counter), olc::WHITE);
        pge.draw_string(pge.screen_width() as i32 - 100, 40, &format!("{}", self.collisions_per_second), olc::WHITE);

        let pos1 = castvec_i32(self.pos1);
        let pos2 = castvec_i32(self.pos2);
        let size1 = castvec_i32(self.size1);
        let size2 = castvec_i32(self.size2);
        
        
        pge.fill_rect_v(pos1, size1, olc::WHITE);
        pge.fill_rect_v(pos2, size2, olc::Pixel::rgb(40, 70, 60));
        true
    }
}
