# Transversal

Template rust application. Rust speed, SPA interactivity.


## Local development

The project provides a local docker compose file to ease to provide a offiline available Oauth2 server, keycloak (see [auth](#openid-oauth-2)).

# Tech choices and architecture

## Backend & Web

### Axum

Lets start by the most simpler part, Axum is part of the tokio project, a thin layer of it & hyper, supports tower. Its blazing fast its simple its predictable its great, provides great base and gets out of your way. There are lower level and faster but also more annoying to work with and not as rich ecosystem.

### OpenID OAuth 2

The authentication system recieves a OpenID client, secret and a "issuer" URL through environment variables, and uses open ID's `.well-known` endpoint to introspect the rest of oauth endpoints. In short, almost any "social login" provider (google, github, etc) should be compatible out of the box as far as you manage to find its issuer URL a client ID and secret. Keycloak is provided as for the simplest one to containerize in a local environment.

### Monitoring & Tracing

Axum is part of the tokio ecosystem so has built-in support to the great tracing crate, rich logging, spans, tracing you name it, built in.

### Sea Session

A more simpler MVC web application allows to the yester-year style of DB persisted sessions, its like cookies but not as annoying or limited. In your controllers you can simply `session.get::<SerializableStruct>().await` the glue code (Axum Session Backend impl) is already in place to persist arbitrary JSON to DB. Out of the box its mainly used to store auth session, but any other Serializable struct can be persisted.

## Frontend & Templating

The main building blocks are UniPoly and beer.css

### **NO BUILD STEP**

This is a quite constraining decision that might be revisited later because there might be a ceiling of functionality that can be achieved without a nodejs build step, but I really wanted to avoid the complexity of managing 2 the projects of different ecosystems and languages. The local DX is already non-ideal (rust compile times) I really rather avoid the extra complexity of having 2 watch process that need to be synchronized.

### Beer.Css

There is nothing revolutionary about it per-se whats important is ticks all the boxes I needed:

- Small size
- "batteries included" components
- accessibility
- no build step

It is missing some quality of life utility classes (e.g. margins) and is quite aggressive overriding meaning of normal HTML tags like `<i>` being now for icons but that characteristic actually helps since when used in raw HTML.

### Unipoly

MVC has a bunch of historic limitations which I think have been overcompensated for with a javascript frontend SPA framework dictatorship killing a lot of the richness exists in computer programming.

Unipoly is part of the SSR renaissance and from a family of increasingly popular libraries of HTML enhancement libraries (the most infamous being HTMX) which the goal of bringing richer SPA characteristics to MVC web apps, JS stateful navigation, in place update of certain parts of the page without full refresh, richer form validation, etc.

It works great it really is a great progressive enhancement library has a great foundation and anything I need seems to be a `up-*` html attribute away no matter how complex the behavior.

### Askama

In brief its Jinja, but for rust, parsed and converted to rust code at compile time and therefore embedded in the final binary. When I was shopping for possible templating solutions I immediately excluded ones that did runtime interpolation/interpretation or anything of the sorts, they just as clunky as Askama templating and still offer a much bigger runtime hit.

Out of the pre-compiled options some were too obscure and/or crazy (see [maud](https://maud.lambda.xyz/)) Askama hit the perfect sweet spot of the battle tested Jinja syntax and blazing speed. But not without its issues

## Known issues

1. Slow compilation times and no auto refresh

   - This is the main tradeoff of Askama, if your templates become rust code when you compile it it means it will go through the infamously slow rust compiler
   - There is currently no mechanism to auto refresh, it is doable but would be a very complicated bunch of code that needed to be selectively disabled by target
   - Askama editor tooling is very weak, altough good documentation and clear errors, still feels awkward your HTML causes rust errors you have to snipe though line numbers

1. The sea-orm Active model <-> serde integration is a little awkward

   - The templates receiving active models to render is a very convenient way to allow per field optional rendering but I got the wrong impression that deserializing into an active model would allow deserialized with missing fields (e.g deserializing a newly created form without id fails)
   - Therefore the JSON value needs a little massaging to be a valid (deserializable) model which might mean inserting a nil `id` field into the JSON value before trying to deserialize it.
   - In the same line, given how HTML forms work everything is a string, even numeric fields, which serde-json natually refuses to auto convert given JSON has numbers, it could've been configured to deserilize anyway but sadly sea-orm is migration-first and you generate entities from DB state, which has limited customization. I settled for something like this in the repository layer:

   ```rust
   json_value["number_prop"] = json!(json_value["number_prop"].as_str().parse::<i32>().unwrap_or_default())
   ```

## Roadmap / Wishlist

- More complex frontend examples
   - More complex Askama examples
   - More complex Unipoly examples
- Replace Askama with Leptos?

## FAQ

- How to generate the files from the database?
  ```bash
  sea generate entity -o models/src/generated --with-serde both --serde-skip-deserializing-primary-key --serde-skip-hidden-column
  ```
