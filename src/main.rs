use std::collections::{
    BTreeMap,
    BTreeSet,
};

extern crate reqwest;

#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

#[derive(Debug, Serialize, Deserialize)]
struct LanguageInfo {
    language_id: i64,

    ace_mode: String,
    color: Option<String>,
    extensions: Option<Vec<String>>,
    tm_scope: Option<String>,

    #[serde(rename = "type")]
    _type: String,
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone)]
struct Color {
    red: i64,
    green: i64,
    blue: i64,
}

impl Color {
    fn from_webcolor(color: &str) -> Self {
        let color = color.trim_start_matches("#");

        let chars = color.chars().collect::<Vec<_>>();
        let mut chars = chars.chunks(2);

        let red = i64::from_str_radix(&char_array_to_string(chars.next().unwrap()), 16).unwrap();
        let green = i64::from_str_radix(&char_array_to_string(chars.next().unwrap()), 16).unwrap();
        let blue = i64::from_str_radix(&char_array_to_string(chars.next().unwrap()), 16).unwrap();

        Self { red, green, blue }
    }

    fn as_webcolor(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }

    fn euclidean_distance(&self, other: &Color) -> f64 {
        let p_red = (other.red - self.red).pow(2);
        let p_green = (other.green - self.green).pow(2);
        let p_blue = (other.blue - self.blue).pow(2);

        ((p_red + p_green + p_blue) as f64).sqrt()
    }
}

fn char_array_to_string(chars: &[char]) -> String {
    chars.iter().fold(String::new(), |mut x, c| {
        x.push(*c);
        x
    })
}

fn main() {
    eprintln!("fetching");

    let body = reqwest::get(
        "https://raw.githubusercontent.com/github/linguist/master/lib/linguist/languages.yml",
    )
    .expect("can not fetch languages from github")
    .text()
    .expect("can not get body from request");

    let languages: BTreeMap<String, LanguageInfo> =
        serde_yaml::from_str(&body).expect("can not deserialize languages");

    let languages_colors: BTreeMap<String, Color> = languages
        .into_iter()
        .filter(|(_, info)| info.color.is_some())
        .map(|(name, info)| {
            let color = Color::from_webcolor(info.color.as_ref().unwrap());
            (name, color)
        })
        .collect();

    let mut used_languages: BTreeSet<String> = BTreeSet::default();
    let mut is_first_color = true;
    let mut nearest_colors: Vec<(String, Color)> = Vec::default();

    eprintln!("sorting");
    for (f_lang, f_color) in languages_colors.clone() {
        let mut shortest_distance = 0.0;
        let mut shortest: Option<(String, Color)> = None;

        for (s_lang, s_color) in &languages_colors {
            if &f_lang == s_lang {
                continue;
            }

            if used_languages.contains(s_lang) {
                continue;
            };

            let distance = f_color.euclidean_distance(s_color);
            if shortest.is_none() || distance < shortest_distance {
                shortest_distance = distance;
                shortest = Some((s_lang.clone(), s_color.clone()));
            }
        }

        if is_first_color {
            used_languages.insert(f_lang.clone());
            nearest_colors.push((f_lang.clone(), f_color.clone()));
            is_first_color = false;
        }

        if shortest.is_some() {
            used_languages.insert(shortest.as_ref().unwrap().0.clone());
            nearest_colors.push(shortest.unwrap());
        }
    }

    let languages_html_name = languages_colors
        .iter()
        .map(|(name, color)| {
            format!(
                r#"<tr class="outline_text">
                    <td bgcolor="{color}">{name}</td>
                    <td bgcolor="{color}"><code>{color}</code></td>
                    </tr>"#,
                name = name,
                color = color.as_webcolor(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let languages_html_nearest = nearest_colors
        .iter()
        .map(|(name, color)| {
            format!(
                r#"<tr class="outline_text">
                    <td bgcolor="{color}">{name}</td>
                    <td bgcolor="{color}"><code>{color}</code></td>
                    </tr>"#,
                name = name,
                color = color.as_webcolor(),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    eprintln!("printing");
    println!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
    <meta charset="utf-8">
    <title>title</title>
    <style>
    body {{
      font-size: 30px
    }}

    tr {{
      line-height: 50px;
    }}

    td {{
      padding-left: 15px;
    }}

    table {{
      width: 100%;
    }}

    .outline_text {{
      color: white;
      text-shadow:
        -1px -1px 0 #000,
        1px -1px 0 #000,
        -1px 1px 0 #000,
        1px 1px 0 #000;
    }}
    </style>
    </head>
    <body>
    <h1>Github Programming Language Colors</h1>

    </h2>By Name</h2>
    <table>
    <tr>
    <th>Language</th><th>Color</th>
    </tr>
    {}
    </table>
    </body>
    </html>

    </h2>By Nearest Color</h2>
    <table>
    <tr>
    <th>Language</th><th>Color</th>
    </tr>
    {}
    </table>
    </body>
    </html>
                 "#,
        languages_html_name, languages_html_nearest
    );
}
