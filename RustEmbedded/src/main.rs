#![no_std]
#![no_main]

// you can put a breakpoint on `rust_begin_unwind` to catch panics
use panic_halt as _;

use cortex_m_rt::entry;

use stm32l4xx_hal::prelude::*;
use stm32l4xx_hal::stm32::{TIM2, DMA1, RCC};

#[entry]
fn main() -> ! {
    // setup the board
    let dp = stm32l4xx_hal::stm32::Peripherals::take().unwrap();

    // setup the peripherals
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr); // sets by default the clock to 16mhz ?!
    let _channels = dp.DMA1.split(&mut rcc.ahb1);

    // setup the LEDs
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
    let mut led = gpiob.pb14.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    
    // setup the PWM
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let c1 = gpioa.pa0.into_push_pull_output(&mut gpioa.moder, &mut gpioa.otyper).into_af1(&mut gpioa.moder, &mut gpioa.afrl);
    let mut pwm = dp.TIM2.pwm(c1, 800.khz(), clocks, &mut rcc.apb1r1);
    let max = pwm.get_max_duty();

    let one_duty = (max * 80 / 125) as u32;
    let zero_duty = (max * 45 / 125) as u32;

    let buffer: [u32; 128] = [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            one_duty, one_duty, one_duty, one_duty, one_duty, one_duty, one_duty, one_duty,
                            zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty,
                            zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty,
                            zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty,
                            zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty, zero_duty,
                            one_duty, one_duty, one_duty, one_duty, one_duty, one_duty, one_duty, one_duty,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0 ];

    pwm.set_duty(0);
    pwm.enable();

    let tim2;
    unsafe {
        tim2 = &*TIM2::ptr(); // general timer 2 pointer
        let rcc_ptr = &*RCC::ptr(); // RCC pointer
        let dma1 = &*DMA1::ptr(); // DMA 1 pointer
    
        rcc_ptr.ahb1enr.modify(|_, w| w.dma1en().set_bit()); // enable DMA1 clock: peripheral clock enable register

        // timer for DMA configuration
        tim2.dier.write(|w| w.tde().set_bit()); // enable DMA trigger
        tim2.dier.write(|w| w.ude().set_bit()); // enable update DMA request
        // tim2.dier.write(|w| w.cc1de().set_bit()); // enable capture/compare 1 DMA request
        // tim2.dier.write(|w| w.uie().set_bit()); // enable update interrupt enable

        let _a = &tim2.ccr1 as *const _ as u32; // very different from 0x4000_0034

        // DMA configuration
        dma1.cselr.write(|w| w.c2s().bits(0b0100)); // set CxS[3:0] to 0100 to map the DMA request to timer 2 channel 1
        dma1.cpar2.write(|w| w.pa().bits(0x4000_0034)); // set the DMA peripheral address register to the capture/compare 1 of TIM2
        dma1.cmar2.write(|w| w.ma().bits(buffer.as_ptr() as u32)); // write the buffer to the memory adress
        dma1.cndtr2.write(|w| w.ndt().bits(buffer.len() as u16)); // number of data to transfer register    
        dma1.ccr2.modify(|_, w| w
            .mem2mem().clear_bit() // memory-to-memory disabled
            .pl().high() // set highest priority
            .msize().bits(2) // size in memory of each transfer: b10 = 32 bits long
            .psize().bits(2) // size of peripheral: b10 = 32 bits long --> 32 or 16 ?? 
            .minc().set_bit() // memory increment mode enabled
            .pinc().clear_bit() // peripheral increment mode disabled
            .circ().clear_bit() // circular mode: the dma transfer is repeated automatically when finished
            .dir().set_bit() // data transfer direction: 1 = read from memory
            .teie().set_bit() // transfer error interrupt enabled
            .htie().set_bit() // half transfer interrupt enabled
            .tcie().set_bit() // transfer complete interrupt enabled
            .en().set_bit() // channel enable
        );
    }

    led.set_high().unwrap();

    loop {
    }
}
