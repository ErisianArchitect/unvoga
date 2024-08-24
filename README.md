An open source Voxel Game/Engine written in Rust with Bevy.

I'll admit, it's not very well written, and it's not complete, but it might give someone some ideas on how to do certain things. I worked pretty hard on this for about a month, then got burnt out, so I figured I'd make it available to the public for use.

To run the "game", `cargo run sandbox`.

When the game loads, there won't be any blocks, so you'll have to modify the code to make blocks appear. There's world generation code present in the engine, but you need to use my [noise editor](https://github.com/ErisianArchitect/noise_editor) to create world generators.

This engine could be modified to become a full game, but right now it's just not there. There's no ambient occlusion, there's no lighting, there's no entity system (for mobs), and there are tons of other features that I had planned but never implemented.

There is incremental loading for the world, but the meshing isn't incremental, so it's fairly slow. I tried making meshing incremental, but I ended up getting some weird bugs that I struggled to fix. My architecture is whack. I dunno, maybe I just have imposter syndrome. I doubt it, I think my code is genuinely bad. I'll let you be the judge!

![Screenshot 2024-08-23 235306](https://github.com/user-attachments/assets/b4e26187-8a9d-48fb-ab96-0c010ad8d0c7)
