import animatedShowcase from "./templates/animated-showcase?raw";
import articleCover from "./templates/article-cover?raw";
import gradientPoster from "./templates/gradient-poster?raw";
import twitterProfileCard from "./templates/twitter-profile-card?raw";
import welcome from "./templates/welcome?raw";

export const templates = [
  {
    id: "welcome",
    name: "Welcome",
    code: welcome,
  },
  {
    id: "twitter-profile-card",
    name: "Twitter Profile Card",
    code: twitterProfileCard,
  },
  {
    id: "article-cover",
    name: "Article Cover",
    code: articleCover,
  },
  {
    id: "gradient-poster",
    name: "Gradient Poster",
    code: gradientPoster,
  },
  {
    id: "keyframe-animation",
    name: "Keyframe Animation",
    code: animatedShowcase,
  },
] as const;

export const defaultTemplate = templates[0].code;
