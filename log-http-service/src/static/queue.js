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
}

var queueApi = new QueueApi('/queue')