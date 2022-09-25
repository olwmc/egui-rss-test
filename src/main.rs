use eframe::egui;
use feed_rs::model::Entry;
use std::collections::HashMap;
mod rss;

fn main() {
    // Make the window resizeable
    let mut options = eframe::NativeOptions::default();
    options.resizable = true;

    // Run the eframe
    eframe::run_native(
        "RSS Reader",
        options,
        Box::new(|_cc| Box::new(RssReader::default())),
    );
}

struct NameUrlPair {
    name: &'static str,
    url:  &'static str,
}

// Our app's data
struct RssReader {
    // List of names and url's that correspond to different feeds
    feed_urls: Vec<NameUrlPair>,

    // Name of feed currently being viewed
    current_feed_url: Option<&'static str>,

    // List of entries of currently viewed feed
    feed_map: HashMap<&'static str, Vec<Entry>>,

    // Error, if any, that has occured
    error: Option<String>,
}

// Helper function to call rss::get_articles and set new feed
impl RssReader {
    // Set the current feed
    fn set_current_feed(&mut self, url: &'static str) -> Result<(), rss::RssError> {
        // If there's no entry in the hasmap
        if self.feed_map.get(url).is_none() {
            // Try to get the articles
            match rss::get_articles(url) {
                // If we've gotten them, set them at the url
                Ok(feed) => {
                    self.feed_map.insert( url, feed.entries );
                },
                // Otherwhise, raise an error
                Err(errtype) => {
                    return Err(errtype)
                },
            }
        }

        Ok(())
    }
}

// Implement default trait for our app
impl Default for RssReader {
    fn default() -> Self {
        Self {
            // Some testing urls and names
            feed_urls: vec![
                NameUrlPair { name: "CNN News",     url: "http://rss.cnn.com/rss/cnn_topstories.rss"},
                NameUrlPair { name: "Broken host",  url: "http://diwdiuwbdiwubdaiubdowqbdqwb.xyz"   },
                NameUrlPair { name: "Not a feed",   url: "https://google.com"},
            ],
            
            // No name at the beginning
            current_feed_url: None,
            
            // No feed at the beginning
            feed_map: HashMap::<&'static str, Vec<Entry>>::new(),

            // No error messages at the start
            error: None,
        }
    }
}

// Our implementation of the App trait for our RssReader struct
impl eframe::App for RssReader {

    // The main update function
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Our top panel which just contains the title "RSS".
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // Center align the title
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("RSS");
            });
        });

        // Storing the old feed name so we can
        let mut name_changed: bool = false;
        
        // Side panel containing list of titles 
        egui::SidePanel::left("Feeds")
            // Set resizeable
            .resizable(true)
            // Show the panel
            .show(&ctx, |ui| {
                // Top heading
                ui.heading("Feed List");

                // For each of the feed urls, make a button
                // with the name and set the onclick behavior
                for feed in &self.feed_urls {
                    if ui.button(feed.name).clicked() {
                        self.current_feed_url = Some(feed.url);
                        name_changed = true;
                    }
                }
        });
        
        // If we have an error, we need to show it before trying to get info for new name
        if !self.error.is_none() {
            // Popup area
            egui::Area::new("popup")
                // We anchor it to the center
                .anchor(egui::Align2::CENTER_TOP, egui::Vec2 {x:0.0, y:0.0})
                // Show it
                .show(&ctx, |ui|{
                    // Within a group
                    ui.group(|ui| {
                        // Label with error contents
                        ui.label(format!("Error: {}", self.error.as_ref().unwrap()));
                        // Okay box
                        if ui.button("Okay").clicked() {
                            // When the box is clicked, reset error status
                            self.error = None;
                        }
                    });
            });

            // Reset current feed name and restart the update loop
            self.current_feed_url = None;
            return;
        }
        
        // If there's no feed after button click check just return
        if self.current_feed_url.is_none() { return; }

        // If name is changed then self.current_feed_url is guaranteed to have Some(),
        // So we set the current_feed here.
        if name_changed {
            // From trying to set the feed, we get a response
            let feed_set_res = self.set_current_feed(self.current_feed_url.unwrap());

            // Match the response
            match feed_set_res {
                // If it's okay, just do nothing
                Ok(()) => {}
                // Else we need to set the error message and reset current_feed_url
                Err(e) => {
                    self.current_feed_url = None;
                    self.error = Some(format!("{}", e));
                    return
                }
            }
        }

        // Central panel holding our rss feed
        egui::CentralPanel::default().show(&ctx, |ui| {
            // We keep it in a scroll area because it can get oversized
            egui::ScrollArea::both().show(ui, |ui| {
                // We use a grid to represent the feed
                egui::Grid::new("feed_list")
                    // Some basic spacing
                    .spacing([5.0, 5.0])
                    // Striped rows are easier to read
                    .striped(true)
                    // Fill up the container
                    .min_col_width(ui.available_width())
                    // Show the widget
                    .show(ui, |ui| {
                        // For each feed_rs::Entry in the list, (convert &Option(Vec<Entry>) to Option(&Vec<Entry>) here)
                        for e in self.feed_map.get(self.current_feed_url.unwrap()).unwrap() {
                            // Decide what to do if there's a publish date
                            match &e.published {
                                Some(d) => { ui.label(format!("[{}] ", d));}
                                None    => { ui.label("[No publish date]"); }
                            }
                            // End the date row
                            ui.end_row();
                            
                            // Link to the article (text of link is set to title if there's any)
                            match &e.title {
                                Some(t) => { ui.hyperlink_to(format!("{}", t.content),        &e.links[0].href); }
                                None    => { ui.hyperlink_to(format!("{}", &e.links[0].href), &e.links[0].href); }
                            }
                            // End the title row
                            ui.end_row();
                        }
                    });
            });
        });
    }
}
