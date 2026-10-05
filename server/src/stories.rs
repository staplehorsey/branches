//! The story kit: the people, pages and small games architects can place in
//! rooms. Everything here is plain data, the same shape a creative architect
//! (an LLM, a person in Claude Code) returns when it invents something new,
//! so authored and invented things live side by side in the world's files.
//!
//! Characters speak in beats: each conversation moves them one line along.
//! Their last lines point somewhere else (another character, another
//! world), which is how stories stitch the houses together. Pages belong to
//! threads and are placed in order, one house at a time, so finding them is
//! a walk.

use crate::procgen::Rng;
use serde_json::{Value, json};

pub struct Cast {
    pub id: &'static str,
    pub name: &'static str,
    pub tags: &'static [&'static str],
    pub coat: &'static str,
    pub accent: &'static str,
    pub hat: &'static str,
    pub lines: &'static [&'static str],
}

pub const CAST: &[Cast] = &[
    Cast {
        id: "odile",
        name: "Odile, the cartographer",
        tags: &["sky", "books", "stone"],
        coat: "#3f5a7a",
        accent: "#f2d58a",
        hat: "wide",
        lines: &[
            "Every house on this street is the same house. I have measured them all twice.",
            "The insides disagree with the outsides. I map the disagreements.",
            "I dropped my pages somewhere between here and the fog. If you find one, read it. They were never meant to be in order.",
            "The map is not finished. It cannot be. Someone keeps building at the edges.",
            "If you meet the boy who counts doors, tell him I found a door that counts back.",
        ],
    },
    Cast {
        id: "pim",
        name: "Pim, who counts doors",
        tags: &["light", "stone", "cozy"],
        coat: "#e98a5a",
        accent: "#ffffff",
        hat: "cap",
        lines: &[
            "Four hundred and twelve. Four hundred and thirteen. You made me lose count.",
            "Doors are easy. It is the doors that are also windows that slow me down.",
            "There is a door in the spawn house that smells like oranges. Do not tell anyone.",
            "Odile says some doors count back. I am not ready for that.",
        ],
    },
    Cast {
        id: "halloway",
        name: "Mrs. Halloway, retired lifeguard",
        tags: &["water"],
        coat: "#d9433a",
        accent: "#ffffff",
        hat: "visor",
        lines: &[
            "No running by the pools. There has never been anyone to tell, but rules are rules.",
            "The water here is always the temperature you were expecting.",
            "Forty years and I never once had to go in after anybody. I go in anyway, on Sundays.",
            "If the house grows another pool, tell the architect to make the shallow end longer.",
        ],
    },
    Cast {
        id: "fen",
        name: "Fen, the gardener",
        tags: &["plants", "water"],
        coat: "#4f7a3a",
        accent: "#e8d27a",
        hat: "straw",
        lines: &[
            "Mind the ferns. They are newer than they look.",
            "I trade seeds with the gardener in the Dusk Orchard. Hers come back pink.",
            "Things grow where people stand still. I have watched it happen.",
            "Plant something in your own house one day. The architect notices.",
        ],
    },
    Cast {
        id: "lumen",
        name: "Lumen, the lamplighter",
        tags: &["light", "neon"],
        coat: "#2b2f45",
        accent: "#ffd27a",
        hat: "tall",
        lines: &[
            "Light a lamp for me? I have three hundred houses tonight and only two hands.",
            "Every lamp you light here stays lit for the next person.",
            "In the Moonlit Meadow I do not light anything. The mushrooms take care of it.",
            "When every lamp in a room is lit, the room remembers. You will see.",
        ],
    },
    Cast {
        id: "ves",
        name: "Ves, the night attendant",
        tags: &["neon", "music"],
        coat: "#7a3fff",
        accent: "#3ff0ff",
        hat: "none",
        lines: &[
            "We are open. We are always open. Nobody has ever asked for change.",
            "The high score belongs to someone called MOTH. I have never seen them play.",
            "If you ring the bells in the right order, the cabinets play a song nobody wrote.",
            "Tell Tobiah the music room is out of tune again. He likes being needed.",
        ],
    },
    Cast {
        id: "coral",
        name: "Auntie Coral",
        tags: &["creatures", "water"],
        coat: "#e86a8a",
        accent: "#9ff0ee",
        hat: "scarf",
        lines: &[
            "Say hello to the jellies. They pretend not to hear, but they do.",
            "I name every fish. I have run out of names twice and started again.",
            "A cat called Moth wanders between the houses. She is welcome everywhere and belongs nowhere.",
            "The Night Aquarium was mine first. Then the house grew it on its own.",
        ],
    },
    Cast {
        id: "saltwick",
        name: "Saltwick, a sculptor",
        tags: &["art", "stone"],
        coat: "#bdb8a8",
        accent: "#3a3a3a",
        hat: "beret",
        lines: &[
            "I carve what the house almost built and then decided against.",
            "This one is called Staircase That Changed Its Mind.",
            "On the salt flats there is a statue nobody made. I am jealous of it.",
            "Leave a note in the visitor book saying you liked the art. It helps more than you think.",
        ],
    },
    Cast {
        id: "juniper",
        name: "Juniper, who listens to snow",
        tags: &["snow", "cozy", "sky"],
        coat: "#9ec7d8",
        accent: "#ffffff",
        hat: "beanie",
        lines: &[
            "Shh. It is snowing indoors again.",
            "Every flake lands somewhere it was meant to. I keep a list.",
            "The fog in Fog Pines is just snow that got shy.",
            "When the house is very quiet you can hear the architect thinking.",
        ],
    },
    Cast {
        id: "tobiah",
        name: "Old Tobiah, piano tuner",
        tags: &["music", "cozy", "art"],
        coat: "#5a3d2a",
        accent: "#d9a63a",
        hat: "bowler",
        lines: &[
            "Every piano in every house is the same piano. Every one is out of tune differently.",
            "Ves sent you? Of course she did.",
            "Middle, high, low. That is the order the bells like. Do not tell the bells I told you.",
            "I tuned a piano in a world that did not exist yet. It exists now. You are welcome.",
        ],
    },
    Cast {
        id: "moth-collector",
        name: "The moth collector",
        tags: &["creatures", "sky", "neon"],
        coat: "#3a3050",
        accent: "#d8ff7a",
        hat: "net",
        lines: &[
            "I do not catch them. I only count them, like Pim with his doors.",
            "The fireflies in the meadow are moths that learned a trick.",
            "There is a cat named after us. We are flattered and a little afraid.",
        ],
    },
];

pub struct Thread {
    pub id: &'static str,
    pub title: &'static str,
    pub tags: &'static [&'static str],
    pub pages: &'static [&'static str],
}

pub const THREADS: &[Thread] = &[
    Thread {
        id: "cartographer",
        title: "The Cartographer's Pages",
        tags: &["books", "sky", "stone"],
        pages: &[
            "Page one. The street runs both ways forever. I chose a direction by flipping a coin, and the coin landed on its edge, so I chose both.",
            "Page two. Every front door is the same door. Every entry hall is a different hall. I have stopped expecting the inside to agree with the outside.",
            "Page three. Behind each house there is a mist where the next room will be. I put my hand into it once. It was warm, like a room someone had just left.",
            "Page four. Some doors lead to other weathers. I walked through one into evening and the evening did not end.",
            "Page five. The far houses are older. Not shabbier, older: the way a photograph is older. The further I walk, the earlier it gets.",
            "Page six. I met a lamplighter who says the houses grow where people linger. I lingered on purpose for an hour. Nothing grew. Then I forgot to watch, and it did.",
            "Page seven. There are worlds that did not exist until someone walked a certain way. I found one shaped exactly like my hesitation.",
            "Page eight. The map cannot be finished. I am going to stop trying and start living in it. If you are reading this, you are already on it.",
        ],
    },
    Thread {
        id: "letters",
        title: "Letters to the Architect",
        tags: &["cozy", "art", "light"],
        pages: &[
            "Dear Architect, please could the next room have a window seat. Not a view, I do not mind what is outside. Just the seat.",
            "Dear Architect, thank you for the fountain. My grandmother would have stood in it.",
            "Dear Architect, someone keeps leaving the lamps lit. Please do not stop them.",
            "Dear Architect, I know you read the visitor books. I know because the ferns appeared the day after I asked.",
            "Dear Architect, is there a room you have never built because nobody asked? Build it anyway. I will come.",
            "Dear Architect, we are leaving this town. Please keep our house the way it was. The far streets will. Thank you for that.",
        ],
    },
    Thread {
        id: "citrus",
        title: "Recipes from the Citrus Kitchen",
        tags: &["plants", "cozy", "light"],
        pages: &[
            "Lemon water. One lemon from the tree in the corner. Water from any pool, they all taste of morning.",
            "Marmalade for a long afternoon. Slice thin, sugar generously, wait longer than seems reasonable.",
            "Orange peel on the radiator. Not to eat. For the house.",
            "Tea for visitors. Steep until they stop talking, then serve.",
        ],
    },
];

pub const POEMS: &[&str] = &[
    "The hallway is longer than the house. / I measured both. / The hallway won.",
    "Somebody lit this lamp before me. / I will leave it lit / for somebody after.",
    "Every door is a question / with the answer already standing / on the other side.",
    "I came for a minute. / The ferns grew an inch. / We are both surprised.",
    "Salt on the floor, sky in the window, / and nobody to tell me / I am late.",
    "If you are lost, good. / Lost is a kind of address / the mailman knows.",
];

/// Which kinds of thing a tag invites.
fn kinds_for(tag: &str) -> &'static [&'static str] {
    match tag {
        "books" => &["page", "page", "character", "poem"],
        "sky" => &["page", "character", "poem"],
        "light" => &["lanterns", "character", "poem"],
        "music" => &["bells", "character"],
        "neon" => &["bells", "character"],
        "creatures" => &["cat", "character"],
        "cozy" => &["character", "page", "poem"],
        "art" => &["character", "poem"],
        "water" => &["character", "poem"],
        "plants" => &["character", "page"],
        "stone" => &["character", "page"],
        "snow" => &["character", "poem"],
        _ => &["poem"],
    }
}

/// What the world already holds, so new things continue stories rather
/// than repeat them.
pub struct WorldStory {
    pub cast_here: Vec<String>,
    pub pages_placed: Vec<(String, usize)>,
    pub cat_here: bool,
}

/// Pick a thing to place for a chamber built around `tag`.
pub fn pick(tag: &str, story: &WorldStory, rng: &mut Rng) -> Option<(String, Value)> {
    let kind = *rng.pick(kinds_for(tag));
    match kind {
        "character" => {
            let fits: Vec<&Cast> = CAST.iter().filter(|c| c.tags.contains(&tag) && !story.cast_here.iter().any(|h| h == c.id)).collect();
            let any: Vec<&Cast> = CAST.iter().filter(|c| !story.cast_here.iter().any(|h| h == c.id)).collect();
            let c = if !fits.is_empty() { *rng.pick(&fits) } else if !any.is_empty() { *rng.pick(&any) } else { return None };
            Some((
                "character".into(),
                json!({ "cast": c.id, "name": c.name, "lines": c.lines, "look": { "coat": c.coat, "accent": c.accent, "hat": c.hat } }),
            ))
        }
        "page" => {
            let threads: Vec<&Thread> = THREADS.iter().filter(|t| t.tags.contains(&tag)).collect();
            let t = if threads.is_empty() { rng.pick(THREADS) } else { *rng.pick(&threads) };
            let next = story.pages_placed.iter().filter(|(id, _)| id == t.id).map(|(_, i)| i + 1).max().unwrap_or(0);
            if next >= t.pages.len() {
                return Some(("note".into(), json!({ "title": "A poem", "text": *rng.pick(POEMS) })));
            }
            Some(("note".into(), json!({ "thread": t.id, "title": t.title, "page": next, "of": t.pages.len(), "text": t.pages[next] })))
        }
        "poem" => Some(("note".into(), json!({ "title": "A poem", "text": *rng.pick(POEMS) }))),
        "lanterns" => Some(("lanterns".into(), json!({ "count": 3 + (rng.next_u64() % 3) as u32, "reward": "When every lamp is lit, the room remembers." }))),
        "bells" => {
            let notes = ["low", "middle", "high"];
            let order: Vec<&str> = (0..3).map(|_| *rng.pick(&notes)).collect();
            Some(("bells".into(), json!({ "order": order, "hint": format!("{} — the order the bells like.", order.join(", ")) })))
        }
        "cat" if !story.cat_here => Some(("cat".into(), json!({ "name": "Moth", "says": "Moth blinks slowly. You have been approved of." }))),
        _ => Some(("note".into(), json!({ "title": "A poem", "text": *rng.pick(POEMS) }))),
    }
}

/// A compact description of the kit, for creative architects' prompts.
pub fn prompt_summary() -> String {
    let cast: Vec<String> = CAST.iter().map(|c| format!("{} ({})", c.name, c.tags.join("/"))).collect();
    let threads: Vec<String> = THREADS.iter().map(|t| format!("{} ({} pages)", t.title, t.pages.len())).collect();
    format!("Cast: {}. Story threads: {}. Game kinds: lanterns, bells, cat.", cast.join("; "), threads.join("; "))
}
