import articleCover from "./templates/article-cover?raw";
import gradientPoster from "./templates/gradient-poster?raw";
import twitterProfileCard from "./templates/twitter-profile-card?raw";
import welcome from "./templates/welcome?raw";

export const templates = [
  {
    name: "Welcome",
    code: welcome,
  },
  {
    name: "Twitter Profile Card",
    code: twitterProfileCard,
  },
  {
    name: "Article Cover",
    code: articleCover,
  },
  {
    name: "Gradient Poster",
    code: gradientPoster,
  },
] as const;

export const defaultTemplate = templates[0].code;
