use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[arg(short, long, value_name = "OWNER", help = "Sets the repository owner")]
    owner: String,

    #[arg(
        short,
        long,
        value_name = "REPO_NAME",
        help = "Sets the repository name"
    )]
    name: String,

    #[arg(short, long, value_name = "COMMIT_HASH", help = "Sets the commit hash")]
    commit: String,

    #[arg(
        short,
        long,
        value_name = "ACCESS_TOKEN",
        help = "Sets the GitHub access token"
    )]
    token: String,
}

#[derive(Debug, Serialize)]
struct QueryVariables {
    owner: String,
    name: String,
    commit: String,
}

#[derive(Debug, Deserialize)]
struct MyResponse {
    data: ResponseData,
}

#[derive(Debug, Deserialize)]
struct ResponseData {
    repository: Repository,
}

#[derive(Debug, Deserialize)]
struct Repository {
    name: String,
    owner: Owner,
    object: Option<Object>,
}

#[derive(Debug, Deserialize)]
struct Owner {
    login: String,
}

#[derive(Debug, Deserialize)]
#[allow(non_snake_case)]
struct Object {
    associatedPullRequests: Option<PullRequests>,
}

#[derive(Debug, Deserialize)]
struct PullRequests {
    nodes: Vec<PullRequestData>,
}

#[derive(Debug, Deserialize)]
struct PullRequestData {
    number: i64,
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    // コマンドライン引数をパース
    let opts: Opts = Opts::parse();

    let variables = QueryVariables {
        owner: opts.owner.clone(),
        name: opts.name.clone(),
        commit: opts.commit.clone(),
    };

    // GraphQLクエリ
    let query = r#"
        query($owner: String!, $name: String!, $commit: String!) {
            repository(owner: $owner, name: $name) {
                name
                owner {
                    login
                }
                object(expression: $commit) {
                    ... on Commit {
                        commitUrl
                        associatedPullRequests(last: 1) {
                            nodes {
                                number
                            }
                        }
                    }
                }
            }
        }
    "#;
    let json_variables_string = serde_json::to_string(&variables).unwrap();
    let json_query = HashMap::from([("query", query), ("variables", &json_variables_string)]);

    // GraphQLクエリを送信して結果を取得
    match request_github_graphql(opts.token, json_query).await {
        Ok(response) => {
            let checked_response = response.error_for_status();
            match checked_response {
                Ok(_res) => {
                    let json_response: MyResponse = _res.json().await?;
                    let pr_url = fetch_github_url(json_response, variables.commit);

                    match pr_url {
                        Some(pr_url) => println!("Github URL: {:?}", pr_url),
                        None => eprintln!("Can not find Pull Request URL"),
                    }
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                }
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            // Handle the error as needed
        }
    }
    Ok(())
}

async fn request_github_graphql(
    token: String,
    json_query: HashMap<&str, &str>,
) -> Result<reqwest::Response, reqwest::Error> {
    // GitHub GraphQLエンドポイント
    let url = "https://api.github.com/graphql";

    let response = reqwest::Client::new()
        .post(url)
        .bearer_auth(token)
        .header(reqwest::header::USER_AGENT, "request")
        .json(&json_query)
        .send()
        .await?;

    Ok(response)
}

fn fetch_github_url(json_response: MyResponse, commit: String) -> Option<String> {
    // レスポンスを解析して必要な情報を取得
    let data = json_response.data;
    let log = &data;
    let repo_owner = &data.repository.owner.login;
    let repo_name = &data.repository.name;
    if let Some(object) = &data.repository.object {
        if let Some(prs) = object.associatedPullRequests.as_ref() {
            if let Some(node) = prs.nodes.first() {
                let pr_number = node.number;
                return Some(format!(
                    "https://github.com/{}/{}/pull/{}",
                    repo_owner, repo_name, pr_number
                ));
            } else {
                return Some(format!(
                    "https://github.com/{}/{}/commit/{}",
                    repo_owner, repo_name, commit
                ));
            }
        }
    }

    eprintln!("{:?}", log);
    return None;
}
