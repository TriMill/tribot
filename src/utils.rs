use rand::Rng;

pub const POLL_COLOR: u32 = 0x225599;
pub const HELP_COLOR: u32 = 0x228844;
pub const WEB_COLOR: u32 = 0x339988;


pub const NUM_EMOJIS: &[&str] = &[
    "0\u{FE0F}\u{20E3}", 
    "1\u{FE0F}\u{20E3}", 
    "2\u{FE0F}\u{20E3}", 
    "3\u{FE0F}\u{20E3}", 
    "4\u{FE0F}\u{20E3}", 
    "5\u{FE0F}\u{20E3}", 
    "6\u{FE0F}\u{20E3}", 
    "7\u{FE0F}\u{20E3}", 
    "8\u{FE0F}\u{20E3}", 
    "9\u{FE0F}\u{20E3}", 
];

pub fn timeformat(mut millis: u64) -> String {
    let mut result = String::new();
    if millis > 60*60*1000 {
        result += &(millis/(60*60*1000)).to_string();
        result += "hr ";
    }
    if millis > 60*1000 {
        result += &((millis/(60*1000))%60).to_string();
        result += "m ";
        millis %= 60*1000;
    }
    result += &format!("{:.3}s", (millis as f64)/1000.);
    result
}

#[derive(Debug, Clone)]
pub struct ErrorBox<T: std::fmt::Debug + Send>(pub T);
impl<T: std::fmt::Debug + Send> std::fmt::Display for ErrorBox<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl<T: std::fmt::Debug + Send> std::error::Error for ErrorBox<T> {}

pub fn roll_dice(dicestr: &str, sort: bool) -> Result<Vec<i64>, &'static str> {
    let repl = dicestr.replace("-", "+-");
    let dice: Vec<&str> = repl.split("+").collect();
    let mut rolls: Vec<i64> = Vec::new();
    let mut rng = rand::thread_rng();
    for die in dice {
        if die.len() == 0 {continue}
        let (die, sign) = if die.chars().next().unwrap() == '-' {
            (&die[1..], -1)
        } else {
            (die, 1)
        };
        let parts = die.split("d").collect::<Vec<&str>>();
        match parts.len() {
            1 => match parts[0].parse::<i64>() {
                Ok(i) => rolls.push(i*sign),
                Err(_) => return Err("could not parse integer")
            },
            2 => {
                let count = match parts[0] {
                    "" => 1,
                    x => match x.parse::<i64>() {
                        Ok(n) => n,
                        Err(_) => return Err("could not parse integer")
                    }
                };
                if count + (rolls.len() as i64) > 2048 {
                    return Err("too many dice")
                }
                let sides = match parts[1].parse::<i64>() {
                    Ok(n) => n,
                    Err(_) => return Err("could not parse integer")
                };
                if sides <= 0 {
                    return Err("dice must have at least one side")
                }
                for _ in 0..count {
                    rolls.push(sign*rng.gen_range(1, sides+1));
                }
            }
            _ => return Err("invalid dice notation")
        }
    }
    if sort {
        rolls.sort();
    }
    Ok(rolls)
}

pub const EIGHT_BALL: &[&str] = &[
    "It is certain.", "It is decidedly so.", "Without a doubt.", 
    "Yes - definitely.", "You may rely on it.", "As I see it, yes.", 
    "Most likely.", "Outlook good.", "Yes.", "Signs point to yes.",

    "Reply hazy, try again.", "Ask again later.", "Better not tell you know.", 
    "Cannot predict now.", "Concentrate and ask again.",

    "Don't count on it.", "My reply is no.", "My sources say no.", 
    "Outlook not so good.", "Very doubtful."
];
pub fn eight_ball() -> &'static str {
    let idx = rand::thread_rng().gen_range(0, EIGHT_BALL.len());
    return EIGHT_BALL[idx];
}


/*
https://en.wikipedia.org/w/api.php?action=query&prop=extracts&exchars=200&explaintext&pageids={{ID}}&format=json

https://en.wikipedia.org/w/api.php?action=query&pageids={{ID}}&prop=pageimages&format=json&pithumbsize=100
*/
const WIKI_API: &str = "https://en.wikipedia.org/w/api.php";
#[derive(Clone, Debug)]
pub struct WikipediaResult {
    pub title: String,
    pub url: String,
    pub text: String,
    pub image_url: Option<String>
}
#[derive(Clone, Debug)]
pub enum WikipediaError {
    Missing(String), Other(String)
}
pub async fn wikipedia(query: &str) -> Result<WikipediaResult, WikipediaError> {
    match wikipedia_inner(query).await {
        Ok(x) => Ok(x),
        Err(e) => {
            match e.downcast::<ErrorBox<&str>>() {
                Ok(s) => Err(WikipediaError::Missing(format!("{}", s))),
                Err(e) => Err(WikipediaError::Other(format!("{}", e)))
            }
        }
    }
}
async fn wikipedia_inner(query: &str) -> Result<WikipediaResult, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let params = [
        ("action", "query"),
        ("list", "search"),
        ("srsearch", query),
        ("srlimit", "1"),
        ("format", "json")
    ];
    let json = client.post(WIKI_API)
        .form(&params)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    let search_count = &json["query"]["searchinfo"]["totalhits"];
    if search_count.as_i64().unwrap_or(0) > 0 {
        let result = &json["query"]["search"][0];
        let id = &result["pageid"].as_i64()
            .ok_or(ErrorBox("No PageId found"))?.to_string();
        let title = result["title"].as_str()
            .ok_or(ErrorBox("No page title found"))?;
        let params_url = [
            ("action", "query"),
            ("prop", "info"),
            ("inprop", "url"),
            ("pageids", id),
            ("format", "json")
        ];
        let params_text = [
            ("action", "query"),
            ("prop", "extracts"),
            ("exchars", "200"),
            ("explaintext", ""),
            ("pageids", id),
            ("format", "json")
        ];
        let params_thumbnail = [
            ("action", "query"),
            ("prop", "pageimages"),
            ("pithumbsize", "720"),
            ("pageids", id),
            ("format", "json")
        ];
        let json_url = client.post(WIKI_API)
            .form(&params_url).send().await?
            .json::<serde_json::Value>().await?;
        let json_text = client.post(WIKI_API)
            .form(&params_text).send().await?
            .json::<serde_json::Value>().await?;
        let json_thumbnail = client.post(WIKI_API)
            .form(&params_thumbnail).send().await?
            .json::<serde_json::Value>().await?;
        let url = json_url["query"]["pages"][id]["canonicalurl"]
            .as_str().ok_or(ErrorBox("Error retrieving page URL"))?;
        let text = json_text["query"]["pages"][id]["extract"]
            .as_str().ok_or(ErrorBox("Error retrieving page extract"))?;
        let image_url = json_thumbnail["query"]["pages"][id]["thumbnail"]["source"].as_str();
        Ok(WikipediaResult {
            title: title.to_owned(),
            url: url.to_owned(),
            text: text.to_owned(),
            image_url: image_url.map(|x| x.to_owned())
        })
    } else {
        Err(ErrorBox("No results found"))?
    }
}
