/*
 * Copyright 2021. gudaoxuri
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use chrono::NaiveDateTime;

#[crud_table(table_name:todo_category)]
#[derive(Clone, Debug)]
pub struct Category {
    pub id: Option<i64>,
    pub name: Option<String>,
}

#[crud_table(table_name:todo_item)]
#[derive(Clone, Debug)]
pub struct Item {
    pub id: Option<i64>,
    pub content: Option<String>,
    pub creator: Option<String>,
    #[serde(skip_serializing)]
    pub create_time: Option<NaiveDateTime>,
    #[serde(skip_serializing)]
    pub update_time: Option<NaiveDateTime>,
    pub category_id: Option<i64>,
}