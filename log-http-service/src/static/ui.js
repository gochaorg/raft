const { createApp, ref, reactive } = Vue

const app = createApp({
    setup() {
      let tabName = localStorage.getItem('tab')
      if( tabName==null ){ tabName = 'entries' }

      let targetServer = localStorage.getItem('target')
      if( targetServer==null ){ targetServer = '' }

      return {
        tabName: ref(tabName),
        count: ref(0),

        ////////////////
        version: ref({
            debug: false,
            name: 'no-name',
            ver: 'no-ver',
        }),
        currentId: ref({
            logId: 'no-log-id',
            blockId: 'no-block-id'
        }),
        lastHeadersNcount: ref(10),
        lastHeadersN: ref([]),
        insertPlainText: ref(''),
        insertPlainId: ref({
            logId: '',
            blockId: '',
        }),

        /////////////
        logFiles: ref([]),

        ///////////
        targetServer: ref(targetServer),
        push: ref({
            logId: '',
            blockId: '',
            log: [],
        }),
        rollUp: ref({
            plan:reactive([]),
            targetId: {
                logId: '',
                blockId: '',
            },
            message: ''
        })
      }
    },
    watch: {
        tabName(cur,old) { localStorage.setItem('tab', cur) },
        targetServer(cur,old) { localStorage.setItem('target',cur) }
    },
    methods: {
        getVersion() {
            queueApi.version().then( d => {
                this.version.debug = d.debug
                this.version.name = d.crate_name
                this.version.ver = d.crate_ver
            });
        },
        getCurrentId() {
            queueApi.currentId().then( d => {
                this.currentId.logId = d.log_id
                this.currentId.blockId = d.block_id
            })
        },
        getLastN() {
            let cnt = this.lastHeadersNcount
            if( typeof(cnt)!="number" ){ 
                cnt = 10 
                this.lastHeadersNcount = cnt
            }
            if( cnt<1 ) {
                cnt = 1
                this.lastHeadersNcount = cnt
            }
            queueApi.lastHeadersN(cnt).then( data => {
                this.lastHeadersN.splice(0)
                data.values.forEach(element => {
                    this.lastHeadersN.push(element)
                });
            })
        },
        postPlainText() {
            queueApi.insertPlain(this.insertPlainText).then(d => {
                this.insertPlainId.logId = d.log_id
                this.insertPlainId.blockId = d.block_id
                this.getLastN()
            })
        },
        getLogFiles() {
            queueApi.files().then(d => {
                this.logFiles.splice(0)
                d.files.forEach(lfile => {
                    this.logFiles.push(lfile)
                })
            })
        },
        switchTail() {
            queueApi.switchTail().then(d => {
                console.log(d)
                this.getLogFiles()
                this.getLastN()
            })
        },
        pushOneRecord() {
            this.push.log.splice(0)
            let targetQueue= new QueueApi(this.targetServer)
            queueApi.rawRecord(this.push.logId, this.push.blockId)
                .then(blob => {
                    this.push.log.push('blob fetched')
                    targetQueue.currentId().then(id => {
                        this.push.log.push('target id fetched')
                        targetQueue.insertRaw(id.log_id, id.block_id, blob).then((r)=>{
                            this.push.log.push('blob inserted '+r)
                            console.log(r)
                        }).catch(e => {
                            this.push.log.push('blob not inserted '+e)
                        })
                    }).catch( e => {
                        this.push.log.push('id not fetched '+e)
                    })
                }).catch(e => {
                    this.push.log.push('blob not fetched '+e)
                })
        },
        rollupBuildPlan() {
            let targetQueue= new QueueApi(this.targetServer)
            this.rollUp.plan.splice(0)
            rollUpLogsPlan(queueApi, targetQueue).then( plan => {
                plan.forEach(r => this.rollUp.plan.push(r))
            }).catch(e => console.log(e))
        },
        rollupExecutePlan(){
            let actions = []
            for(let i=0;i<this.rollUp.plan.length;i++){
                actions.push(this.rollUp.plan[i])
            }
            (async () => {
                while(actions.length>0){
                    let act = actions.shift()
                    try {
                        let res = await act.execute(act)                    
                        act.state = 'succ'
                        act.log.push(res)
                    } catch(err) {
                        act.state = 'fail'
                        act.log.push(err)
                        break
                    }
                }
            })()
        }
    }    
  }).mount('#app')

window.addEventListener('load', (e)=>{
    app.getVersion()
    app.getCurrentId()
    app.getLastN()
    app.getLogFiles()
});
