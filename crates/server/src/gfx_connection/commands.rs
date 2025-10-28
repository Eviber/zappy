use {
    crate::{client::ClientError, player::PlayerId, state::state},
    alloc::string::String,
    core::{fmt::Write, str::FromStr, time::Duration},
};

/// Handles a single command received from the client.
pub async fn handle_one_command(fd: ft::Fd, command: &[u8]) -> Result<(), ClientError> {
    let mut tokens = command.split(u8::is_ascii_whitespace);
    let mut buffer = String::new();

    // Parse the command.
    let command = match tokens.next() {
        None | Some(b"") => return Ok(()),
        Some(command) => command,
    };

    match command {
        // Map size
        //
        // EXAMPLE: msz      ->       msz WIDTH HEIGHT
        //
        // Returns the width and height of the map.
        b"msz" => {
            let st = state();
            _ = writeln!(buffer, "msz {} {}", st.world.width, st.world.height);
            ft_async::futures::write_all(fd, buffer.as_bytes()).await?;
            Ok(())
        }

        // Contents of a map tile
        //
        // EXAMPLE: bct X Y     ->     bct X Y q q q q q q q
        //
        // Returns the contents of a specific tile.
        b"bct" => {
            let Some(x) = parse_token::<u32>(tokens.next()) else {
                _ = writeln!(buffer, "error: can't parse X");
                return Ok(());
            };
            let Some(y) = parse_token::<u32>(tokens.next()) else {
                _ = writeln!(buffer, "error: can't parse Y");
                return Ok(());
            };
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                return Ok(());
            }
            let st = state();
            if x >= st.world.width {
                _ = writeln!(
                    buffer,
                    "error: X coordinate is out of bound (width is {}, X is {})",
                    st.world.width, x,
                );
                return Ok(());
            }
            if y >= st.world.height {
                _ = writeln!(
                    buffer,
                    "error: Y coordinate is out of bound (height is {}, Y is {})",
                    st.world.height, y,
                );
                return Ok(());
            }
            let cell = st.world.cells[y as usize * st.world.width as usize + x as usize];
            _ = writeln!(
                buffer,
                "bct {} {} {} {} {} {} {} {} {}",
                x,
                y,
                cell.food,
                cell.linemate,
                cell.deraumere,
                cell.sibur,
                cell.mendiane,
                cell.phiras,
                cell.thystame,
            );
            ft_async::futures::write_all(fd, buffer.as_bytes()).await?;

            Ok(())
        }

        // Entire map contents
        //
        // EXAMPLE: mct          ->      bct 0 0 q q q q q q q\nbct 0 1 q q q q q q q ...
        b"mct" => {
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                ft_async::futures::write_all(fd, buffer.as_bytes()).await?;
                return Ok(());
            }

            let st = state();
            for y in 0..st.world.height {
                for x in 0..st.world.width {
                    let cell = st.world.cells[y as usize * st.world.width as usize + x as usize];
                    _ = writeln!(
                        buffer,
                        "bct {} {} {} {} {} {} {} {} {}",
                        x,
                        y,
                        cell.food,
                        cell.linemate,
                        cell.deraumere,
                        cell.sibur,
                        cell.mendiane,
                        cell.phiras,
                        cell.thystame,
                    );
                }
            }
            drop(st);

            ft_async::futures::write_all(fd, buffer.as_bytes()).await?;
            Ok(())
        }

        // Tcam names
        //
        // EXAMPLE: tna          -> tna <name1>\ntna <name2> ...
        //
        // Returns the name of all active teams.
        b"tna" => {
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                return Ok(());
            }
            let st = state();
            buffer.push_str("tna ");
            for team in &st.teams {
                buffer.push_str(&team.name);
                buffer.push('\n');
            }
            ft_async::futures::write_all(fd, buffer.as_bytes()).await?;
            Ok(())
        }

        // Player position
        //
        // EXAMPLE: ppo <player_id>       ->       ppo <player_id> X Y O
        //
        // Returns the position and orientation of a particular player.
        b"ppo" => {
            let player_id = match parse_token::<PlayerId>(tokens.next()) {
                Some(id) => id,
                None => {
                    _ = writeln!(buffer, "error: can't parse player id");
                    fd.async_write_all(buffer.as_ref()).await?;
                    return Ok(());
                }
            };
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            }

            let st = state();
            let Some(player) = st.players.get(player_id) else {
                _ = writeln!(buffer, "error: player not found");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            };

            _ = writeln!(
                buffer,
                "ppo {} {} {} {}",
                player_id, player.x, player.y, player.facing,
            );
            fd.async_write_all(buffer.as_ref()).await?;

            Ok(())
        }

        // Player level
        //
        // EXAMPLE: plv <player_id>       ->       plv <player_id> LEVEL
        //
        // Returns the level of a particular player.
        b"plv" => {
            let player_id = match parse_token::<PlayerId>(tokens.next()) {
                Some(id) => id,
                None => {
                    _ = writeln!(buffer, "error: can't parse player id");
                    fd.async_write_all(buffer.as_ref()).await?;
                    return Ok(());
                }
            };
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            }

            let st = state();
            let Some(_player) = st.players.get(player_id) else {
                _ = writeln!(buffer, "error: player not found");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            };

            // TODO: Send the actual level of the player once it is stored in the
            // global state.
            _ = writeln!(buffer, "plv {} {}", player_id, 1);
            fd.async_write_all(buffer.as_ref()).await?;
            Ok(())
        }

        // Player inventory
        //
        // EXAMPLE: pin <player_id>       ->       pin <player_id> q q q q q q q
        //
        // Returns the inventory of a particular player.
        b"pin" => {
            let player_id = match parse_token::<PlayerId>(tokens.next()) {
                Some(id) => id,
                None => {
                    _ = writeln!(buffer, "error: can't parse player id");
                    fd.async_write_all(buffer.as_ref()).await?;
                    return Ok(());
                }
            };
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            }

            let st = state();
            let Some(_player) = st.players.get(player_id) else {
                _ = writeln!(buffer, "error: player not found");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            };

            // TODO: Send the actual inventory of the player once it is stored in the
            // global state.
            _ = writeln!(
                buffer,
                "pin {} {} {} {} {} {} {} {}",
                player_id, 1, 1, 1, 1, 1, 1, 1,
            );
            fd.async_write_all(buffer.as_ref()).await?;
            Ok(())
        }

        // Request for current time unit
        //
        // EXAMPLE: sgt            ->       sgt TIME_UNIT
        //
        // Returns the current time unit.
        b"sgt" => {
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            }

            // TODO: Implement time unit retrieval from the global state once it is
            // implemented.
            _ = writeln!(buffer, "sgt {}", state().tick_duration.as_secs_f32());
            fd.async_write_all(buffer.as_ref()).await?;

            Ok(())
        }

        // Time unit modification
        //
        // EXAMPLE: sst <time_unit>       ->       sgt <time_unit>
        //
        // Modifies the current time unit.
        b"sst" => {
            let Some(new_time_unit) = parse_token::<f32>(tokens.next()) else {
                _ = writeln!(buffer, "error: invalid time unit");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            };
            if tokens.next().is_some() {
                _ = writeln!(buffer, "error: too many arguments");
                fd.async_write_all(buffer.as_ref()).await?;
                return Ok(());
            }

            {
                let mut st = state();
                st.tick_duration = Duration::from_secs_f32(new_time_unit);
                _ = writeln!(buffer, "sgt {}", st.tick_duration.as_secs_f32());
            }

            fd.async_write_all(buffer.as_ref()).await?;

            Ok(())
        }

        _ => {
            _ = writeln!(buffer, "error: unknown command");
            fd.async_write_all(buffer.as_ref()).await?;
            Ok(())
        }
    }
}

/// Parses a token into a value of type `T`.
fn parse_token<T: FromStr>(maybe_token: Option<&[u8]>) -> Option<T> {
    let token = maybe_token?;
    let s = core::str::from_utf8(token).ok()?;
    let value = T::from_str(s).ok()?;
    Some(value)
}
