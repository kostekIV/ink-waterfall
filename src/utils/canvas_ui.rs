// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use fantoccini::{
    Client,
    ClientBuilder,
    Locator,
};
use regex::Regex;
use serde_json::{
    self,
    map::Map,
    value::Value,
};

pub struct CanvasUI {
    pub client: Client,
}

impl CanvasUI {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Connect to webdriver instance that is listening on port 4444
        let client = ClientBuilder::native()
            .capabilities(get_capabilities())
            .connect("http://localhost:4444")
            .await?;
        Ok(Self { client })
    }

    pub async fn close(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.client.close().await?;
        Ok(())
    }

    pub async fn upload(
        &mut self,
        path: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.client
            .goto("https://paritytech.github.io/canvas-ui/#/upload")
            .await?;

        eprintln!("click action button");
        self.client
            .wait_for_find(Locator::Css(".actions button"))
            .await?
            .click()
            .await?;

        eprintln!("click settings");
        self.client
            .find(Locator::Css(".app--SideBar-settings"))
            .await?
            .click()
            .await?;

        eprintln!("click local node");
        self.client
            .find(Locator::XPath("//*[contains(text(),'Local Node')]"))
            .await?
            .click()
            .await?;

        eprintln!("click upload");
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Upload & Instantiate Contract')]",
            ))
            .await?
            .click()
            .await?;
        let inject = String::from(
            "(function (){\
        var d = document;\
        if (!d.getElementById('jquery')) {\
            var s = d.createElement('script');\
            s.src = 'https://code.jquery.com/jquery-3.6.0.min.js';\
            s.id = 'jquery';\
            d.body.appendChild(s);\
            (function() {\
              var nTimer = setInterval(function() {\
                if (window.jQuery) {\
                    $('body').append('<div id=\"jquery-ready\"></div');\
                  clearInterval(nTimer);\
                }\
              }, 100);\
            })();\
        }\
    })();",
        );
        self.client.execute(&*inject, Vec::new()).await?;

        self.client
            .wait_for_find(Locator::Css("#jquery-ready"))
            .await?;

        eprintln!("click combobox");
        self.client
            .execute("$('[role=combobox]').click()", Vec::new())
            .await?;

        eprintln!("click alice");
        self.client
            .execute("$('[name=alice]').click()", Vec::new())
            .await?;

        let mut upload = self
            .client
            .find(Locator::Css(".ui--InputFile input"))
            .await?;
        upload
            //.send_keys("/ci-cache/ink-waterfall/targets/master/run/ink/flipper.contract")
            .send_keys(path)
            .await?;
        self.client
            .execute("$(\".ui--InputFile input\").trigger('change')", Vec::new())
            .await?;

        eprintln!("click details");
        self.client
            .execute(
                "$(\":contains('Constructor Details')\").click()",
                Vec::new(),
            )
            .await?;

        eprintln!("click instantiate");
        self.client
            .execute("$(\"button:contains('Instantiate')\").click()", Vec::new())
            .await?;

        eprintln!("click sign and submit");
        self.client
            .execute(
                "$(\"button:contains('Sign & Submit')\").click()",
                Vec::new(),
            )
            .await?;

        // h1: Contract successfully instantiated
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Contract successfully instantiated')]",
            ))
            .await?;

        eprintln!("click dismiss");
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss all notifications')]",
            ))
            .await?
            .click()
            .await?;

        // wait for disappearance animation to finish instead
        // otherwise the notifications might occlude buttons
        eprintln!("wait for animation to finish");
        self.client
            .execute("$('.ui--Status').hide()", Vec::new())
            .await?;

        eprintln!("click execute");
        self.client
            .find(Locator::XPath(
                "//button[contains(text(),'Execute Contract')]",
            ))
            .await?
            .click()
            .await?;

        let url = self.client.current_url().await?;

        let re = Regex::new(
            r"https://paritytech.github.io/canvas-ui/#/execute/([0-9a-zA-Z]+)/0",
        )
        .expect("invalid regex");
        let captures = re.captures(url.as_str()).expect("must exist");
        let addr = captures.get(1).expect("no capture group").as_str();
        log::info!("addr {:?}", addr);
        Ok(String::from(addr))
    }

    pub async fn execute_rpc(
        &mut self,
        addr: &str,
        method: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let url = format!(
            "https://paritytech.github.io/canvas-ui/#/execute/{}/0",
            addr
        );
        self.client.goto(url.as_str()).await?;

        // open listbox for methods
        eprintln!("click listbox");
        self.client
            .find(Locator::XPath(
                "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
            ))
            .await?
            .click()
            .await?;

        // click `method`
        eprintln!("choose {:?}", method);
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'{}')]", method);
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // click call
        eprintln!("click call");
        self.client
            .find(Locator::XPath("//button[contains(text(),'Call')]"))
            .await?
            .click()
            .await?;

        // must contain false
        let mut el = self.client.wait_for_find(Locator::XPath("//div[@class = 'outcomes']/*[1]//div[@class = 'ui--output monospace']/div[1]")).await?;
        let txt = el.text().await?;
        log::info!("value {:?}", txt);
        Ok(txt)
    }

    pub async fn execute_transaction(
        &mut self,
        addr: &str,
        method: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!(
            "https://paritytech.github.io/canvas-ui/#/execute/{}/0",
            addr
        );
        self.client.goto(url.as_str()).await?;

        // open listbox for methods
        eprintln!("click listbox");
        self.client
            .find(Locator::XPath(
                "//*[contains(text(),'Message to Send')]/ancestor::div[1]/div",
            ))
            .await?
            .click()
            .await?;

        // click `method`
        eprintln!("choose {:?}", method);
        let path = format!("//*[contains(text(),'Message to Send')]/ancestor::div[1]/div//*[contains(text(),'{}')]", method);
        self.client
            .find(Locator::XPath(&path))
            .await?
            .click()
            .await?;

        // click call
        eprintln!("click call");
        self.client
            .find(Locator::XPath("//button[contains(text(),'Call')]"))
            .await?
            .click()
            .await?;

        // wait for notification to show up
        self.client
            .wait_for_find(Locator::XPath(
                "//div[@class = 'status' and contains(text(), 'queued')]",
            ))
            .await?;

        // click sign and submit
        eprintln!("sign and submit");
        self.client
            .find(Locator::XPath("//button[contains(text(),'Sign & Submit')]"))
            .await?
            .click()
            .await?;

        // maybe assert?
        eprintln!("waiting for success notification");
        self.client.wait_for_find(Locator::XPath("//div[@class = 'status']/ancestor::div/div[@class = 'header' and contains(text(), 'ExtrinsicSuccess')]")).await?;
        self.client
            .wait_for_find(Locator::XPath(
                "//*[contains(text(),'Dismiss all notifications')]",
            ))
            .await?
            .click()
            .await?;

        // clear all
        eprintln!("click clear all");
        self.client
            .find(Locator::XPath("//*[text() = 'Clear all']"))
            .await?
            .click()
            .await?;

        // let mut el = self.client.wait_for_find(Locator::XPath("//div[@class = 'outcomes']/*[1]//div[@class = 'ui--output monospace']/div[1]")).await?;
        // let txt = el.text().await?;
        // Ok(txt)
        // eprintln!("value transaction {:?}", value);
        Ok(())
    }
}

#[cfg(feature = "headless")]
fn get_capabilities() -> Map<String, Value> {
    let mut caps = Map::new();
    let opts = serde_json::json!({ "args": ["--headless"] });
    caps.insert("moz:firefoxOptions".to_string(), opts.clone());
    caps
}

#[cfg(not(feature = "headless"))]
fn get_capabilities() -> Map<String, Value> {
    Map::new()
}
