use crate::theme::ChartTheme;
use crate::types::CandleData;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

pub struct ChartController {
    pub container: HtmlElement,
    pub theme: ChartTheme,
}

impl ChartController {
    pub fn new(container: HtmlElement, theme: Option<ChartTheme>) -> Result<Self, JsValue> {
        let th = theme.unwrap_or_default();
        Ok(Self {
            container,
            theme: th,
        })
    }

    pub fn set_data(&self, _candles: &[CandleData]) -> Result<(), JsValue> {
        // Interacts with vendored Lightweight Charts via JS bridge
        Ok(())
    }

    pub fn update_candle(&self, _candle: &CandleData) -> Result<(), JsValue> {
        Ok(())
    }

    pub fn resize(&self, _width: u32, _height: u32) -> Result<(), JsValue> {
        Ok(())
    }
}
