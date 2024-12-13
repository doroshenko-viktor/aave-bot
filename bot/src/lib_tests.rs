use bigdecimal::BigDecimal;

#[test]
pub fn test_1() {
    let x1 = BigDecimal::from(10);
    let x2 = BigDecimal::from(5);

    let sum = &x1 + &x2;
    let dub = &x1 - &x2;
    let pow = &x1 * &x2;
    let div = &x1 / &x2;

    println!("sum:{}; dub: {}; pow: {}; div: {}", sum, dub, pow, div);
}
