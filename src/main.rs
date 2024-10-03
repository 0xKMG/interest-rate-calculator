#[macro_use] extern crate rocket;

use rocket::form::Form;
use rocket::response::content::RawHtml;
use fixed::types::I80F48;
use fixed_macro::types::I80F48;

mod constants;
use constants::*;

pub struct Market {
    pub total_supply_assets: I80F48,  // Total supply in assets (fixed-point)
    pub total_borrow_assets: I80F48,  // Total borrowed assets (fixed-point)
    pub last_update: u64,             // Timestamp of the last update (unused in current context)
}

#[derive(FromForm)]
struct InputData {
    current_utilization: f64,
    elapsed_time_seconds: i64,
    curve_steepness: f64,
    initial_rate: f64,
    adjustment_speed: f64,
    target_utilization: f64,
    min_rate: f64,
    max_rate: f64,
}

fn get_html_content() -> String {
    format!(r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Interest Rate Calculator</title>
            <style>
                body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #f0f0f0; }}
                .container {{ max-width: 600px; margin: 0 auto; background-color: white; padding: 20px; border-radius: 8px; box-shadow: 0 0 10px rgba(0,0,0,0.1); }}
                h1 {{ color: #333; text-align: center; }}
                form {{ margin: 20px 0; }}
                label {{ display: block; margin: 10px 0 5px; color: #666; }}
                input {{ padding: 8px; width: 100%; box-sizing: border-box; border: 1px solid #ddd; border-radius: 4px; }}
                button {{ padding: 10px 15px; margin-top: 10px; background-color: #4CAF50; color: white; border: none; border-radius: 4px; cursor: pointer; width: 100%; }}
                button:hover {{ background-color: #45a049; }}
                #results {{ margin-top: 20px; background-color: #e9f7ef; padding: 15px; border-radius: 4px; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Interest Rate Calculator</h1>
                <form id="calculatorForm">
                    <label for="current_utilization">Current Utilization Ratio (in %):</label>
                    <input type="number" id="current_utilization" name="current_utilization" required min="0" max="100" step="0.01">
                    
                    <label for="elapsed_time_seconds">Elapsed Time (in seconds):</label>
                    <input type="number" id="elapsed_time_seconds" name="elapsed_time_seconds" required min="0">
                    
                    <label for="curve_steepness">Curve Steepness:</label>
                    <input type="number" id="curve_steepness" name="curve_steepness" value="4" step="0.01">
                    
                    <label for="initial_rate">Initial Rate (% per year):</label>
                    <input type="number" id="initial_rate" name="initial_rate" value="4" step="0.01">
                    
                    <label for="adjustment_speed">Adjustment Speed (% per year):</label>
                    <input type="number" id="adjustment_speed" name="adjustment_speed" value="50" step="0.01">
                    
                    <label for="target_utilization">Target Utilization (%):</label>
                    <input type="number" id="target_utilization" name="target_utilization" value="90" step="0.01">
                    
                    <label for="min_rate">Minimum Rate (% per year):</label>
                    <input type="number" id="min_rate" name="min_rate" value="0.1" step="0.01">
                    
                    <label for="max_rate">Maximum Rate (% per year):</label>
                    <input type="number" id="max_rate" name="max_rate" value="200" step="0.01">
                    
                    <button type="submit">Calculate</button>
                    <button type="button" id="useDefault">Use Default Config</button>
                </form>
                <div id="results"></div>
            </div>
            <script>
                document.getElementById('calculatorForm').addEventListener('submit', function(e) {{
                    e.preventDefault();
                    const formData = new FormData(this);
                    fetch('/calculate', {{
                        method: 'POST',
                        body: formData
                    }})
                    .then(response => response.text())
                    .then(data => {{
                        document.getElementById('results').innerHTML = data;
                    }})
                    .catch(error => console.error('Error:', error));
                }});

                document.getElementById('useDefault').addEventListener('click', function() {{
                    document.getElementById('curve_steepness').value = '4';
                    document.getElementById('initial_rate').value = '4';
                    document.getElementById('adjustment_speed').value = '50';
                    document.getElementById('target_utilization').value = '90';
                    document.getElementById('min_rate').value = '0.1';
                    document.getElementById('max_rate').value = '200';
                }});
            </script>
        </body>
    </html>
    "#)
}

#[get("/")]
fn index() -> RawHtml<String> {
    RawHtml(get_html_content())
}

#[post("/calculate", data = "<input_data>")]
fn calculate(input_data: Form<InputData>) -> String {
    let current_utilization = I80F48::from_num(input_data.current_utilization / 100.0);
    let elapsed_time_seconds = input_data.elapsed_time_seconds;

    let curve_steepness = I80F48::from_num(input_data.curve_steepness);
    let initial_rate = I80F48::from_num(input_data.initial_rate / 100.0) / SECONDS_PER_YEAR;
    let adjustment_speed = I80F48::from_num(input_data.adjustment_speed) / SECONDS_PER_YEAR;
    let target_utilization = I80F48::from_num(input_data.target_utilization / 100.0);
    let min_rate = I80F48::from_num(input_data.min_rate/ 100.0) / SECONDS_PER_YEAR;
    let max_rate = I80F48::from_num(input_data.max_rate/ 100.0) / SECONDS_PER_YEAR;

    let start_rate_at_target = initial_rate;
    let (avg_rate_at_target, end_rate_at_target) = calc_avg_and_end_rate(
        start_rate_at_target,
        elapsed_time_seconds,
        current_utilization,
        curve_steepness,
        adjustment_speed,
        target_utilization,
        min_rate,
        max_rate
    );

    format!(
        r#"<h2>Results</h2>
        <p><strong>Average Rate before Applying Curve (APY):</strong> {:.2}%</p>
        <p><strong>Average Rate after Applying Curve (APY):</strong> {:.2}%</p>"#,
        rate_per_second_to_rate_per_year(avg_rate_at_target) * I80F48!(100),
        rate_per_second_to_rate_per_year(end_rate_at_target) * I80F48!(100)
    )
}


pub fn rate_per_second_to_rate_per_year(rate_per_second: I80F48) -> I80F48 {
    rate_per_second * SECONDS_PER_YEAR
}


pub fn calc_avg_and_end_rate(
    start_rate_at_target: I80F48,
    elapsed_time_seconds: i64,
    utilization: I80F48,
    curve_steepness: I80F48,
    adjustment_speed: I80F48,
    target_utilization: I80F48,
    min_rate: I80F48,
    max_rate: I80F48,
) -> (I80F48, I80F48) {
    // Calculate utilization ratio
    // Normalization factor for error calculation
    let err_norm_factor = if utilization > target_utilization {
        I80F48::ONE - target_utilization
    } else {
        target_utilization
    };

    // Error between current utilization and target utilization
    let err = (utilization - target_utilization) / err_norm_factor;

    let (avg_rate_at_target, end_rate_at_target) = if start_rate_at_target == I80F48::ZERO {
        // If starting rate is zero, initialize rates
        (start_rate_at_target, start_rate_at_target)
    } else {
        // Calculate speed of adjustment
        let speed = adjustment_speed * err;
        let linear_adaptation = speed * I80F48::from_num(elapsed_time_seconds);

        // Calculate end and mid rates at target
        let end_rate_at_target = _new_rate_at_target(start_rate_at_target, linear_adaptation, min_rate, max_rate);
        let mid_rate_at_target = _new_rate_at_target(
            start_rate_at_target,
            linear_adaptation / I80F48::from_num(2),
            min_rate,
            max_rate,
        );
        let avg_rate_at_target = (start_rate_at_target
            + end_rate_at_target
            + I80F48::from_num(2) * mid_rate_at_target)
            / I80F48::from_num(4);

        // Log rates (converted to annual percentage rates)
        // println!(
        //     "Start Rate (APY): {}%",
        //     rate_per_second_to_rate_per_year(start_rate_at_target) * I80F48!(100)
        // );
        // println!(
        //     "End Rate at Target (APY): {}%",
        //     rate_per_second_to_rate_per_year(end_rate_at_target) * I80F48!(100)
        // );
        // 
        (avg_rate_at_target, end_rate_at_target)
    };

    let avg_rate_after_curve = curve(avg_rate_at_target, err, curve_steepness);

    (avg_rate_at_target, avg_rate_after_curve)
}

fn _new_rate_at_target(
    start_rate_at_target: I80F48,
    linear_adaptation: I80F48,
    min_rate: I80F48,
    max_rate: I80F48,
) -> I80F48 {
    // // Log linear adaptation value
    // println!("Linear Adaptation: {}", linear_adaptation);
    // Compute exponent as a floating-point number
    let exponent = linear_adaptation.to_num::<f64>();
    /* Clamp exponent to prevent overflow,  consideration as below: 
    exp(50.0) ≈ 5.1847055e21  =>  within I80F48 range 2^80
    exp(-50.0) ≈ 8.2218062e-22 => within I80F48 range 2^-80
     */
    // also the cap is more than enough for practical purposes, as the rate is capping at 200% per year.
    let max_exponent = 50.0;  
    let min_exponent = -50.0; 
    let clamped_exponent = exponent.max(min_exponent).min(max_exponent);
    let w_exp = I80F48::from_num(clamped_exponent.exp());

    // println!("Exponential Adjustment Factor: {}", w_exp);

    // Calculate new rate at target
    let exp_value = start_rate_at_target * w_exp;

    let clamped_value = exp_value
        .max(min_rate)
        .min(max_rate);
    // Return the clamped rate
    clamped_value
}

pub fn curve(rate_at_target: I80F48, err: I80F48, curve_steepness: I80F48) -> I80F48 {
    // Coefficient calculation
    let coeff = if err < I80F48::ZERO {
        I80F48::ONE - I80F48::ONE / curve_steepness
    } else {
        curve_steepness - I80F48::ONE
    };

    // Calculate the rate
    let result = (coeff * err + I80F48::ONE) * rate_at_target;

    result
}




#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index, calculate])
}