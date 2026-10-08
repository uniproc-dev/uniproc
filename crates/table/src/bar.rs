#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bar {
    pub thumb: f32,
    pub travel: f32,
}

pub fn bar(content: f32, viewport: f32, track: f32, min_thumb: f32) -> Option<Bar> {
    if content <= viewport || track <= 0.0 {
        return None;
    }
    let thumb = (viewport / content * track).max(min_thumb).min(track);
    Some(Bar { thumb, travel: track - thumb })
}

impl Bar {
    pub fn top(self, position: f32, max_position: f32) -> f32 {
        if max_position <= 0.0 {
            return 0.0;
        }
        (position / max_position).clamp(0.0, 1.0) * self.travel
    }

    pub fn position(self, top: f32, max_position: f32) -> f32 {
        if self.travel <= 0.0 {
            return 0.0;
        }
        (top / self.travel).clamp(0.0, 1.0) * max_position
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_that_fits_has_no_bar() {
        assert_eq!(bar(300.0, 320.0, 316.0, 24.0), None);
        assert_eq!(bar(320.0, 320.0, 316.0, 24.0), None);
    }

    #[test]
    fn the_thumb_is_the_share_of_the_content_in_view() {
        assert_eq!(bar(1280.0, 320.0, 400.0, 24.0), Some(Bar { thumb: 100.0, travel: 300.0 }));
    }

    #[test]
    fn the_thumb_never_gets_shorter_than_its_minimum() {
        assert_eq!(bar(64000.0, 320.0, 400.0, 24.0), Some(Bar { thumb: 24.0, travel: 376.0 }));
    }

    #[test]
    fn the_thumb_travels_with_the_position() {
        let bar = Bar { thumb: 100.0, travel: 300.0 };
        assert_eq!(bar.top(0.0, 960.0), 0.0);
        assert_eq!(bar.top(480.0, 960.0), 150.0);
        assert_eq!(bar.top(960.0, 960.0), 300.0);
    }

    #[test]
    fn a_dragged_thumb_gives_back_the_position_and_stops_at_the_ends() {
        let bar = Bar { thumb: 100.0, travel: 300.0 };
        assert_eq!(bar.position(150.0, 960.0), 480.0);
        assert_eq!(bar.position(-40.0, 960.0), 0.0);
        assert_eq!(bar.position(900.0, 960.0), 960.0);
    }
}
