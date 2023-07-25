class QueueApi {
    constructor(base) {
        this.base = base
    }
    
    /** 
     * @returns promise with { debug: string, crate_name: string, crate_ver: string }
     */
    version() {
        return fetch(this.base + '/version').then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** @return promise with { log_id: string, rec_id: string }  */
    currentId() {
        return fetch(this.base + '/tail/id').then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** 
     * fetch last N records 
     * 
     * @returns promise with 
     * @example
     * 
     *     {
     *       "values": [
     *         {
     *           "rid": {
     *             "log_id": "0",
     *             "block_id": "3"
     *           },
     *           "result": {
     *             "Succ": {
     *               "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
     *               "log_id": "0",
     *               "block_id": "3",
     *               "options": {
     *                 "mime": "text/plain",
     *                 "encoding": "utf-8",
     *                 "time": "2023-07-21T22:37:38.481669+00:00"
     *               },
     *               "position": "380",
     *               "head_size": 135,
     *               "data_size": 7,
     *               "tail_size": 8,
     *               "preview": "bla bla"
     *             }
     *           }
     *         },
     *         {
     *           "rid": {
     *             "log_id": "0",
     *             "block_id": "2"
     *           },
     *           "result": {
     *             "Succ": {
     *               "log_file": "/home/user/code/rust/raft/log-http-service/app_data/queue/2023-07-21T18-29-39-a9p29.binlog",
     *               "log_id": "0",
     *               "block_id": "2",
     *               "options": {
     *                 "time": "2023-07-21T22:29:36.327761+00:00",
     *                 "encoding": "utf-8",
     *                 "mime": "text/plain"
     *               },
     *               "position": "231",
     *               "head_size": 135,
     *               "data_size": 6,
     *               "tail_size": 8,
     *               "preview": "hello2"
     *             }
     *           }
     *         }
     *       ]
     *     }
     */
    lastHeadersN(count) {
        return fetch(this.base + '/headers/last/'+count).then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }

    /** 
     * insert plain text at end of log 
     * 
     * @returns promise with { log_id: string, block_id: string }
    */
    insertPlain(text) {
        return fetch(this.base + '/insert/plain', {
            method: 'POST',
            headers: {
                'Content-Type': 'text/plain'
            },
            cache: 'no-cache',
            body: text
        }).then(res => {
            if( res.ok ){
                return res.json()
            }else{
                return Promise.reject("not succ executred")
            }
        })
    }
}

var queueApi = new QueueApi('/queue')