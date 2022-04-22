import {spawnSync} from  "child_process";

export type DetectorType = "double-lock" | "conflict-lock";

export function runkDetector(cargoDir: string, dType: DetectorType): string {
    const newEnv = Object.assign({}, process.env);
    const child = spawnSync(
        `cd ${cargoDir} && cargo clean && cargo +nightly-2022-01-27 lock-bug-detect ${dType}`, 
        {shell: true, env: newEnv}
    );
    return child.stdout.toString();
}


// VSCode -> extension -> LSP -> cargo check (our code) -> parse stdout -> return LSP receiver


export function parseDetectorOutput(s: string) {
    let lines = s.split("\n");
    // state:
    // 0: start
    // 1: first lock type
    // 2: first lock pos
    // 3: second lock type
    // 4: second lock pos
    //5: call chain
    let state = 0;
    let results = [];
    let result = Object();
    for (let i in lines) {
        let line = lines[i];
        if (state == 0) {
            if (line.startsWith("{")) {
                let regex = /FirstLock: \((\w+), "(.+)"\)/;
                let matchObj = line.match(regex);
                if (matchObj) {
                    result["firstLock"] = {type: `${matchObj[1]}<${matchObj[2]}>`};
                    state = 1;
                }
            }
        }
        else if (state == 1) {
            let regex = /([^\t].*?):(\d+:\d+: \d+:\d+)/;
            let matchObj = line.match(regex);
            if (matchObj) {
                let fname = matchObj[1];
                let pos = matchObj[2];
                result["firstLock"].pos = pos;
                result["firstLock"].fname = fname;
                result["firstLock"].msg = "the other lock causing double-lock.";
                state = 2;
            }
        }
        else if (state == 2) {
            let regex = /SecondLock: \((\w+), "(.+)"\)/;
            let matchObj = line.match(regex);
            if (matchObj) {
                result["secondLock"] = {type: `${matchObj[1]}<${matchObj[2]}>`};
                state = 3;
            }
        }
        else if (state == 3) {
            let regex = /([^\t].*?):(\d+:\d+: \d+:\d+)/;
            let matchObj = line.match(regex);
            if (matchObj) {
                let fname = matchObj[1];
                let pos = matchObj[2];
                result["secondLock"].pos = pos;
                result["secondLock"].fname = fname;
                result["secondLock"].msg = "Potential double-locking bug.";
                state = 4;
            }
        }
        else if (state == 4) {
            let regex = /Callchains: \{(.+)\}/;
            let matchObj = line.match(regex);
            if (matchObj) {
                result["firstLock"].msg += " Call chain: " + matchObj[1];
                state = 0;
                results.push(result);
                result = Object();
            }
        }
    }
    return results;
}
