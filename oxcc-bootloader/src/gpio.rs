use stm32f7::stm32f7x7;

/// Set up GPIOs for ethernet.
///
/// All GPIO clocks are already enabled.
pub fn gpio_init(peripherals: &mut stm32f7x7::Peripherals) {
    let gpioa = &peripherals.GPIOA;
    let gpiob = &peripherals.GPIOB;
    let gpioc = &peripherals.GPIOC;
    let gpiog = &peripherals.GPIOG;

    // Status LED (red) on PB14
    gpiob.moder.modify(|_, w| w.moder14().output());
    gpiob.odr.modify(|_, w| w.odr14().set_bit());

    // User button on PC13, pull-down/active-high
    gpioc.moder.modify(|_, w| w.moder13().input());
    gpioc.pupdr.modify(|_, w| w.pupdr13().pull_down());

    // Configure ethernet related GPIO:
    // GPIOA 1, 2, 7
    // GPIOB 13
    // GPIOC 1, 4, 5
    // GPIOG 2, 11, 13
    // All set to AF11 and very high speed
    gpioa.moder.modify(|_, w| {
        w.moder1()
            .alternate()
            .moder2()
            .alternate()
            .moder7()
            .alternate()
    });
    gpiob.moder.modify(|_, w| w.moder13().alternate());
    gpioc.moder.modify(|_, w| {
        w.moder1()
            .alternate()
            .moder4()
            .alternate()
            .moder5()
            .alternate()
    });
    gpiog.moder.modify(|_, w| {
        w.moder2()
            .alternate()
            .moder11()
            .alternate()
            .moder13()
            .alternate()
    });
    gpioa.ospeedr.modify(|_, w| {
        w.ospeedr1()
            .very_high_speed()
            .ospeedr2()
            .very_high_speed()
            .ospeedr7()
            .very_high_speed()
    });
    gpiob.ospeedr.modify(|_, w| w.ospeedr13().very_high_speed());
    gpioc.ospeedr.modify(|_, w| {
        w.ospeedr1()
            .very_high_speed()
            .ospeedr4()
            .very_high_speed()
            .ospeedr5()
            .very_high_speed()
    });
    gpiog.ospeedr.modify(|_, w| {
        w.ospeedr2()
            .very_high_speed()
            .ospeedr11()
            .very_high_speed()
            .ospeedr13()
            .very_high_speed()
    });
    gpioa
        .afrl
        .modify(|_, w| w.afrl1().af11().afrl2().af11().afrl7().af11());
    gpiob.afrh.modify(|_, w| w.afrh13().af11());
    gpioc
        .afrl
        .modify(|_, w| w.afrl1().af11().afrl4().af11().afrl5().af11());
    gpiog.afrl.modify(|_, w| w.afrl2().af11());
    gpiog.afrh.modify(|_, w| w.afrh11().af11().afrh13().af11());
}
